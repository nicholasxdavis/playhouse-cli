use crate::cmd::r#async as async_cmd;
use crate::config::load_settings;
use crate::engines::metrics_util::{error_metrics, finalize_metrics};
use crate::install;
use crate::report;
use crate::tools;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct TrivyOptions {
    /// Clear Trivy scan cache before running (used by verify).
    pub clear_cache: bool,
}

pub async fn execute(workspace: &str, options: &TrivyOptions) -> (i32, Value) {
    if load_settings().auto_install_tools {
        if let Err(e) = install::ensure_trivy(true).await {
            let metrics = error_metrics("trivy", 5, &e, json!({ "fix": "playhouse install" }));
            let _ = report::save_engine_report(workspace, "trivy", &metrics);
            return (5, metrics);
        }
    }

    let trivy = tools::trivy_program();
    let check = async_cmd(&trivy).arg("--version").output().await;
    if !matches!(check, Ok(o) if o.status.success()) {
        let metrics = error_metrics("trivy", 5, "trivy not found", json!({ "fix": "playhouse install" }));
        let _ = report::save_engine_report(workspace, "trivy", &metrics);
        return (5, metrics);
    }

    let settings = load_settings();
    let severity = settings.trivy_severity.replace(' ', "");
    let scan = crate::workspace::scan_root_str(workspace);
    let cache_dir = tools::playhouse_dir(workspace).join("cache").join("trivy");
    let _ = std::fs::create_dir_all(&cache_dir);

    if options.clear_cache {
        let _ = async_cmd(&trivy)
            .args([
                "clean",
                "--scan-cache",
                "--cache-dir",
                &cache_dir.to_string_lossy(),
            ])
            .output()
            .await;
    }

    let skip_dirs = crate::workspace::trivy_skip_dirs(workspace);
    let main = run_trivy_pass(&trivy, &scan, ".", &severity, &cache_dir, &skip_dirs).await;

    let well_known = Path::new(&scan).join(".well-known");
    let merged = if well_known.is_dir() {
        let dot = run_trivy_pass(
            &trivy,
            &scan,
            ".well-known",
            &severity,
            &cache_dir,
            &skip_dirs,
        )
        .await;
        match (main, dot) {
            (Ok((main_data, tool_exit)), Ok((dot_data, _))) => {
                Ok((merge_trivy_results(&main_data, &dot_data), tool_exit))
            }
            (Ok(pair), Err(_)) => Ok(pair),
            (Err(_e), Ok((dot_data, tool_exit))) => Ok((dot_data, tool_exit)),
            (Err(e), Err(_)) => Err(e),
        }
    } else {
        main
    };

    match merged {
        Ok((trivy_data, tool_exit)) => {
            if trivy_data.is_null() {
                let metrics = error_metrics("trivy", 5, "trivy JSON was null", json!({}));
                let _ = report::save_engine_report(workspace, "trivy", &metrics);
                return (5, metrics);
            }

            let (vuln_count, secret_count) = count_findings(&trivy_data);
            let findings_ok = vuln_count == 0 && secret_count == 0;
            let code = if !findings_ok {
                4
            } else if tool_exit != 0 {
                5
            } else {
                0
            };
            let passed = code == 0;
            let metrics = finalize_metrics(
                code,
                if tool_exit != code {
                    Some(tool_exit)
                } else {
                    None
                },
                json!({
                    "engine": "trivy",
                    "passed": passed,
                    "scanComplete": true,
                    "summary": {
                        "vulnerabilities": vuln_count,
                        "secrets": secret_count,
                    },
                    "raw": trivy_data,
                }),
            );
            let _ = report::save_engine_report(workspace, "trivy", &metrics);
            (code, metrics)
        }
        Err(e) => {
            let metrics = error_metrics("trivy", 5, &e, json!({}));
            let _ = report::save_engine_report(workspace, "trivy", &metrics);
            (5, metrics)
        }
    }
}

pub async fn run(workspace: &str, json: bool, quiet: bool) -> i32 {
    let (code, metrics) = execute(workspace, &TrivyOptions::default()).await;

    if !quiet {
        if json {
            println!("{}", serde_json::to_string_pretty(&metrics).unwrap_or_default());
        } else if let Some(err) = metrics.get("error").and_then(|e| e.as_str()) {
            eprintln!("[x] {err}");
        } else {
            let vuln_count = metrics["summary"]["vulnerabilities"].as_u64().unwrap_or(0);
            let secret_count = metrics["summary"]["secrets"].as_u64().unwrap_or(0);
            if code == 0 {
                println!("[*] Trivy: No high or critical vulnerabilities or secrets found");
            } else {
                println!("Trivy Security Scan:");
                if vuln_count > 0 {
                    println!("  [x] {vuln_count} high/critical vulnerabilities found");
                }
                if secret_count > 0 {
                    println!("  [x] {secret_count} secrets detected");
                }
            }
        }
    }
    code
}

async fn run_trivy_pass(
    trivy: &str,
    scan_dir: &str,
    target: &str,
    severity: &str,
    cache_dir: &PathBuf,
    skip_dirs: &[String],
) -> Result<(Value, i32), String> {
    let mut cmd = async_cmd(trivy);
    cmd.args([
        "fs",
        "--scanners",
        "vuln,secret",
        "--severity",
        severity,
        "--format",
        "json",
        "--quiet",
        "--cache-dir",
        &cache_dir.to_string_lossy(),
    ]);
    for dir in skip_dirs {
        cmd.arg("--skip-dirs").arg(dir);
    }
    cmd.arg(target).current_dir(scan_dir);

    let out = crate::cmd::output_with_timeout(&mut cmd)
        .await
        .map_err(|e| format!("Failed to run trivy: {e}"))?;

    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let tool_exit = out.status.code().unwrap_or(1);

    if stdout.is_empty() {
        return Err("trivy returned empty output".into());
    }

    let data: Value =
        serde_json::from_str(&stdout).map_err(|e| format!("failed to parse trivy JSON: {e}"))?;
    Ok((data, tool_exit))
}

fn merge_trivy_results(main: &Value, extra: &Value) -> Value {
    let mut merged = main.clone();
    let Some(main_results) = merged.get_mut("Results").and_then(|r| r.as_array_mut()) else {
        return extra.clone();
    };
    let Some(extra_results) = extra.get("Results").and_then(|r| r.as_array()) else {
        return merged;
    };
    for result in extra_results {
        if let Some(target) = result.get("Target").and_then(|t| t.as_str()) {
            if let Some(existing) = main_results
                .iter_mut()
                .find(|r| r.get("Target").and_then(|t| t.as_str()) == Some(target))
            {
                merge_result_entry(existing, result);
            } else {
                main_results.push(result.clone());
            }
        } else {
            main_results.push(result.clone());
        }
    }
    merged
}

fn merge_result_entry(dst: &mut Value, src: &Value) {
    let Some(dst_map) = dst.as_object_mut() else {
        return;
    };
    let Some(src_map) = src.as_object() else {
        return;
    };
    for key in ["Vulnerabilities", "Secrets"] {
        if let Some(src_items) = src_map.get(key).and_then(|v| v.as_array()) {
            let entry = dst_map
                .entry(key.to_string())
                .or_insert_with(|| json!([]));
            if let Some(dst_items) = entry.as_array_mut() {
                for item in src_items {
                    if !dst_items.contains(item) {
                        dst_items.push(item.clone());
                    }
                }
            }
        }
    }
}

pub fn count_findings(data: &Value) -> (u64, u64) {
    let mut vulns = 0u64;
    let mut secrets = 0u64;

    if let Some(results) = data.get("Results").and_then(|r| r.as_array()) {
        for result in results {
            if let Some(vulnerabilities) = result.get("Vulnerabilities").and_then(|v| v.as_array()) {
                vulns += vulnerabilities.len() as u64;
            }
            if let Some(found_secrets) = result.get("Secrets").and_then(|s| s.as_array()) {
                secrets += found_secrets.len() as u64;
            }
        }
    }

    (vulns, secrets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_trivy_results_combines_secrets() {
        let main = json!({
            "Results": [{
                "Target": ".",
                "Secrets": [{ "RuleID": "a" }]
            }]
        });
        let extra = json!({
            "Results": [{
                "Target": ".well-known",
                "Secrets": [{ "RuleID": "b" }]
            }]
        });
        let merged = merge_trivy_results(&main, &extra);
        let results = merged["Results"].as_array().unwrap();
        assert_eq!(results.len(), 2);
    }
}
