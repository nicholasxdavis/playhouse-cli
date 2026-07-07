use crate::cmd::r#async as async_cmd;
use crate::config::load_settings;
use crate::engines::metrics_util::{error_metrics, finalize_metrics};
use crate::install;
use crate::pkgmgr::{self, PackageManager};
use crate::report;
use crate::tools;
use serde_json::{json, Value};
use std::path::Path;

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
    let cache_dir = crate::config::playhouse_home().join("cache").join("trivy");
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
    let scan_path = Path::new(&scan);
    let lockfile_missing = scan_path.join("package.json").is_file()
        && !pkgmgr::has_node_lockfile(scan_path);

    let merged = run_trivy_pass(&trivy, &scan, ".", &severity, &cache_dir, &skip_dirs).await;

    match merged {
        Ok((mut trivy_data, tool_exit)) => {
            let mut scan_path_used = json!("trivy-fs");
            let mut pm_audit: Option<Value> = None;
            let mut dependency_warning: Option<String> = None;

            if lockfile_missing {
                dependency_warning = Some(
                    "No Node lockfile found — dependency scan may be incomplete; running package manager audit fallback"
                        .into(),
                );
                let pm = PackageManager::resolve(workspace, &settings.package_manager);
                match pm.audit_high_critical(scan_path).await {
                    Ok((pm_count, audit_data)) => {
                        scan_path_used = json!("pm-audit-fallback");
                        pm_audit = Some(audit_data);
                        if pm_count > 0 {
                            merge_pm_audit_warning(&mut trivy_data, pm_count);
                        }
                    }
                    Err(e) => {
                        dependency_warning = Some(format!(
                            "No lockfile and PM audit fallback failed: {e}"
                        ));
                    }
                }
            }

            if trivy_data.is_null() {
                let metrics = error_metrics("trivy", 5, "trivy JSON was null", json!({}));
                let _ = report::save_engine_report(workspace, "trivy", &metrics);
                return (5, metrics);
            }

            let (vuln_count, secret_count) = count_findings(&trivy_data);
            let incomplete = lockfile_missing && pm_audit.is_none();
            let findings_ok = vuln_count == 0 && secret_count == 0 && !incomplete;
            let code = if incomplete || !findings_ok {
                4
            } else if tool_exit != 0 {
                5
            } else {
                0
            };
            let passed = code == 0;
            let mut body = json!({
                "engine": "trivy",
                "passed": passed,
                "scanComplete": !incomplete,
                "scanPath": scan_path_used,
                "lockfileMissing": lockfile_missing,
                "summary": {
                    "vulnerabilities": vuln_count,
                    "secrets": secret_count,
                },
                "raw": trivy_data,
            });
            if let Some(w) = dependency_warning {
                body["dependencyWarning"] = json!(w);
            }
            if let Some(a) = pm_audit {
                body["pmAudit"] = a;
            }
            let metrics = finalize_metrics(
                code,
                if tool_exit != code {
                    Some(tool_exit)
                } else {
                    None
                },
                body,
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
    cache_dir: &Path,
    skip_dirs: &[String],
) -> Result<(Value, i32), String> {
    let report_path =
        std::env::temp_dir().join(format!("playhouse-trivy-{}.json", std::process::id()));
    let mut cmd = async_cmd(trivy);
    for arg in build_trivy_fs_args(severity, cache_dir, skip_dirs, &report_path, target) {
        cmd.arg(arg);
    }
    cmd.current_dir(scan_dir);

    let out = crate::cmd::output_with_timeout(&mut cmd)
        .await
        .map_err(|e| format!("Failed to run trivy: {e}"))?;

    let tool_exit = out.status.code().unwrap_or(1);
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();

    let json_text = std::fs::read_to_string(&report_path).map_err(|e| {
        let detail = if stderr.is_empty() {
            e.to_string()
        } else {
            format!("{e}; trivy stderr: {stderr}")
        };
        format!("trivy report missing at {}: {detail}", report_path.display())
    })?;
    let _ = std::fs::remove_file(&report_path);

    parse_trivy_json(&json_text).map(|data| (data, tool_exit))
}

/// Build `trivy fs` arguments. Uses `--output` + `--quiet` (Trivy 0.72+); never `--hidden` or `--log-level`.
fn build_trivy_fs_args(
    severity: &str,
    cache_dir: &Path,
    skip_dirs: &[String],
    output_path: &Path,
    target: &str,
) -> Vec<String> {
    let mut args = vec![
        "fs".into(),
        "--scanners".into(),
        "vuln,secret".into(),
        "--severity".into(),
        severity.into(),
        "--format".into(),
        "json".into(),
        "--quiet".into(),
        "--cache-dir".into(),
        cache_dir.to_string_lossy().into_owned(),
        "--output".into(),
        output_path.to_string_lossy().into_owned(),
    ];
    for dir in skip_dirs {
        args.push("--skip-dirs".into());
        args.push(dir.clone());
    }
    args.push(target.into());
    args
}

fn parse_trivy_json(json_text: &str) -> Result<Value, String> {
    let json_text = json_text.trim();
    if json_text.is_empty() {
        return Err("trivy returned empty output".into());
    }
    serde_json::from_str(json_text).map_err(|e| format!("failed to parse trivy JSON: {e}"))
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

fn merge_pm_audit_warning(data: &mut Value, pm_count: u64) {
    if let Some(obj) = data.as_object_mut() {
        obj.insert(
            "PmAuditFindings".into(),
            json!({ "count": pm_count, "source": "package-manager-audit" }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const FIXTURE_JSON: &str = r#"{
        "SchemaVersion": 2,
        "Results": [
            {
                "Target": "Cargo.lock",
                "Class": "lang-pkgs",
                "Vulnerabilities": [
                    {
                        "VulnerabilityID": "CVE-2024-0001",
                        "Severity": "HIGH"
                    }
                ]
            },
            {
                "Target": "config.env",
                "Class": "secret",
                "Secrets": [
                    {
                        "RuleID": "aws-access-key-id",
                        "Severity": "CRITICAL"
                    }
                ]
            }
        ]
    }"#;

    #[test]
    fn trivy_fs_args_use_output_not_deprecated_flags() {
        let cache = PathBuf::from("/tmp/trivy-cache");
        let output = PathBuf::from("/tmp/playhouse-trivy-out.json");
        let args = build_trivy_fs_args("HIGH,CRITICAL", &cache, &["node_modules".into()], &output, ".");

        assert!(args.contains(&"--output".into()));
        assert!(args.contains(&"--quiet".into()));
        assert!(args.contains(&"json".into()));
        assert!(!args.iter().any(|a| a == "--hidden"));
        assert!(!args.iter().any(|a| a == "--log-level"));
        assert!(args.contains(&"--skip-dirs".into()));
        assert!(args.contains(&"node_modules".into()));
    }

    #[test]
    fn parse_trivy_fixture_json() {
        let data = parse_trivy_json(FIXTURE_JSON).expect("fixture should parse");
        assert!(data.get("Results").and_then(|r| r.as_array()).is_some());
    }

    #[test]
    fn parse_trivy_json_rejects_empty() {
        assert!(parse_trivy_json("  ").is_err());
    }

    #[test]
    fn count_findings_from_fixture() {
        let data = parse_trivy_json(FIXTURE_JSON).unwrap();
        let (vulns, secrets) = count_findings(&data);
        assert_eq!(vulns, 1);
        assert_eq!(secrets, 1);
    }

    #[test]
    fn count_findings_empty_results() {
        let data = json!({ "Results": [] });
        assert_eq!(count_findings(&data), (0, 0));
    }
}