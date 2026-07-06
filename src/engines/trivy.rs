use crate::cmd::r#async as async_cmd;
use crate::config::load_settings;
use crate::install;
use crate::report;
use crate::tools;

pub async fn execute(workspace: &str) -> (i32, serde_json::Value) {
    if load_settings().auto_install_tools {
        if let Err(e) = install::ensure_trivy(true).await {
            let metrics = serde_json::json!({
                "engine": "trivy",
                "error": e,
                "fix": "playhouse install",
                "passed": false,
            });
            let _ = report::save_engine_report(workspace, "trivy", &metrics);
            return (5, metrics);
        }
    }

    let trivy = tools::trivy_program();
    let check = async_cmd(&trivy).arg("--version").output().await;
    if !matches!(check, Ok(o) if o.status.success()) {
        let metrics = serde_json::json!({
            "engine": "trivy",
            "error": "trivy not found",
            "fix": "playhouse install",
            "passed": false,
        });
        let _ = report::save_engine_report(workspace, "trivy", &metrics);
        return (5, metrics);
    }

    let settings = load_settings();
    let severity = settings.trivy_severity.replace(' ', "");

    let scan = crate::workspace::scan_root_str(workspace);

    let result = {
        let mut cmd = async_cmd(&trivy);
        cmd.args([
                "fs",
                "--scanners",
                "vuln,secret",
                "--severity",
                &severity,
                "--format",
                "json",
                "--quiet",
                ".",
            ])
            .current_dir(&scan);
        crate::cmd::output_with_timeout(&mut cmd).await
    };

    match result {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let exit = out.status.code().unwrap_or(1);

            if stdout.is_empty() {
                let metrics = serde_json::json!({
                    "engine": "trivy",
                    "error": "trivy returned empty output",
                    "passed": false,
                    "exitCode": exit,
                });
                let _ = report::save_engine_report(workspace, "trivy", &metrics);
                return (5, metrics);
            }

            let trivy_data: serde_json::Value = match serde_json::from_str(&stdout) {
                Ok(v) => v,
                Err(e) => {
                    let metrics = serde_json::json!({
                        "engine": "trivy",
                        "error": format!("failed to parse trivy JSON: {e}"),
                        "passed": false,
                        "exitCode": exit,
                    });
                    let _ = report::save_engine_report(workspace, "trivy", &metrics);
                    return (5, metrics);
                }
            };

            if trivy_data.is_null() {
                let metrics = serde_json::json!({
                    "engine": "trivy",
                    "error": "trivy JSON was null",
                    "passed": false,
                    "exitCode": exit,
                });
                let _ = report::save_engine_report(workspace, "trivy", &metrics);
                return (5, metrics);
            }

            let (vuln_count, secret_count) = count_findings(&trivy_data);
            let findings_ok = vuln_count == 0 && secret_count == 0;
            let passed = findings_ok && exit == 0;
            let code = if !findings_ok {
                4
            } else if exit != 0 {
                5
            } else {
                0
            };
            let metrics = serde_json::json!({
                "engine": "trivy",
                "passed": passed,
                "exitCode": exit,
                "summary": {
                    "vulnerabilities": vuln_count,
                    "secrets": secret_count,
                },
                "raw": trivy_data,
            });
            let _ = report::save_engine_report(workspace, "trivy", &metrics);
            (code, metrics)
        }
        Err(e) => {
            let metrics = serde_json::json!({
                "engine": "trivy",
                "error": format!("Failed to run trivy: {e}"),
                "passed": false,
            });
            let _ = report::save_engine_report(workspace, "trivy", &metrics);
            (5, metrics)
        }
    }
}

pub async fn run(workspace: &str, json: bool, quiet: bool) -> i32 {
    let (code, metrics) = execute(workspace).await;

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

fn count_findings(data: &serde_json::Value) -> (u64, u64) {
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
