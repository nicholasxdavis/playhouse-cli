use std::path::Path;

use crate::cmd::r#async as async_cmd;
use crate::config::load_settings;
use crate::install;
use crate::report;
use crate::tools;

pub async fn execute(url: &str, workspace: &str) -> (i32, serde_json::Value) {
    if load_settings().auto_install_tools {
        if let Err(e) = install::ensure_arkenar(true).await {
            let metrics = serde_json::json!({
                "engine": "arkenar",
                "error": e,
                "fix": "playhouse install",
                "passed": false,
            });
            let _ = report::save_engine_report(workspace, "arkenar", &metrics);
            return (5, metrics);
        }
    }

    let program = tools::arkenar_program();
    let check = async_cmd(&program).arg("--help").output().await;
    if !matches!(check, Ok(o) if o.status.success()) {
        let metrics = serde_json::json!({
            "engine": "arkenar",
            "error": "arkenar not installed",
            "fix": "playhouse install",
            "passed": false,
        });
        let _ = report::save_engine_report(workspace, "arkenar", &metrics);
        return (5, metrics);
    }

    let settings = load_settings();
    let report_dir = tools::playhouse_dir(workspace).join("reports");
    let _ = std::fs::create_dir_all(&report_dir);
    let output_path = report_dir.join("arkenar.json");

    let mode = if settings.arkenar_advanced_mode {
        "advanced"
    } else {
        "simple"
    };

    let mut args = vec![
        url.to_string(),
        "-m".into(),
        mode.into(),
        "-o".into(),
        output_path.to_string_lossy().into_owned(),
        "--scope".into(),
        "--rate-limit".into(),
        settings.arkenar_rate_limit.to_string(),
        "--crawler-max-urls".into(),
        settings.arkenar_max_urls.to_string(),
    ];

    if settings.arkenar_param_fuzz {
        args.push("--enable-param-fuzz".into());
    }
    if settings.arkenar_js_analysis {
        args.push("--enable-js-analysis".into());
    }

    let result = async_cmd(&program).args(&args).output().await;

    match result {
        Ok(out) => {
            let exit = out.status.code().unwrap_or(1);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let findings = parse_report(&output_path).or_else(|| parse_stdout_json(&stdout));

            if findings.is_none() {
                let metrics = serde_json::json!({
                    "engine": "arkenar",
                    "passed": false,
                    "reportParseError": true,
                    "target": url,
                    "reportPath": output_path,
                    "exitCode": exit,
                    "error": if output_path.is_file() {
                        "failed to parse arkenar report"
                    } else {
                        "arkenar report file missing"
                    },
                });
                let _ = report::save_engine_report(workspace, "arkenar", &metrics);
                return (if exit != 0 { exit } else { 5 }, metrics);
            }

            let data = findings.unwrap();
            let (high, medium, low, total) = summarize_findings(&data);
            let threshold_fail = high > 0 || medium > 0;
            let passed = !threshold_fail && exit == 0;
            let code = if threshold_fail {
                3
            } else if exit != 0 {
                1
            } else {
                0
            };
            let metrics = serde_json::json!({
                "engine": "arkenar",
                "passed": passed,
                "target": url,
                "reportPath": output_path,
                "summary": {
                    "high": high,
                    "medium": medium,
                    "low": low,
                    "total": total,
                },
                "raw": data,
            });
            let _ = report::save_engine_report(workspace, "arkenar", &metrics);
            (code, metrics)
        }
        Err(e) => {
            let metrics = serde_json::json!({
                "engine": "arkenar",
                "error": format!("Failed to run arkenar: {e}"),
                "passed": false,
            });
            let _ = report::save_engine_report(workspace, "arkenar", &metrics);
            (5, metrics)
        }
    }
}

pub async fn run(url: &str, workspace: &str, json: bool, quiet: bool) -> i32 {
    let (code, metrics) = execute(url, workspace).await;

    if !quiet {
        if json {
            println!("{}", serde_json::to_string_pretty(&metrics).unwrap_or_default());
        } else if let Some(err) = metrics.get("error").and_then(|e| e.as_str()) {
            eprintln!("[x] {err}");
        } else {
            let high = metrics["summary"]["high"].as_u64().unwrap_or(0);
            let medium = metrics["summary"]["medium"].as_u64().unwrap_or(0);
            let low = metrics["summary"]["low"].as_u64().unwrap_or(0);
            let report_path = metrics["reportPath"].as_str().unwrap_or(".playhouse/reports/arkenar.json");
            println!("Arkenar DAST Scan: {url}");
            println!("  High:   {high}");
            println!("  Medium: {medium}");
            println!("  Low:    {low}");
            println!("  Report: {report_path}");
            if code == 0 {
                println!("[*] No high/medium findings");
            } else if code == 3 {
                println!("[x] High/medium findings detected");
            } else {
                println!("[x] Arkenar scan failed");
            }
        }
    }
    code
}

fn parse_report(path: &Path) -> Option<serde_json::Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn parse_stdout_json(stdout: &str) -> Option<serde_json::Value> {
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            if let Ok(v) = serde_json::from_str(trimmed) {
                return Some(v);
            }
        }
    }
    None
}

fn summarize_findings(data: &serde_json::Value) -> (u64, u64, u64, u64) {
    let mut high = 0u64;
    let mut medium = 0u64;
    let mut low = 0u64;

    walk_findings(data, &mut |severity| {
        let s = severity.to_lowercase();
        if s.contains("high") || s.contains("critical") {
            high += 1;
        } else if s.contains("medium") {
            medium += 1;
        } else {
            low += 1;
        }
    });

    let total = high + medium + low;
    (high, medium, low, total)
}

fn walk_findings(value: &serde_json::Value, on_severity: &mut dyn FnMut(&str)) {
    match value {
        serde_json::Value::Array(arr) => {
            for item in arr {
                walk_findings(item, on_severity);
            }
        }
        serde_json::Value::Object(map) => {
            if let Some(sev) = map
                .get("severity")
                .or_else(|| map.get("risk"))
                .or_else(|| map.get("level"))
                .and_then(|v| v.as_str())
            {
                on_severity(sev);
            }
            if let Some(findings) = map.get("findings").or_else(|| map.get("results")) {
                walk_findings(findings, on_severity);
            }
            for (_, v) in map {
                if v.is_array() || v.is_object() {
                    walk_findings(v, on_severity);
                }
            }
        }
        _ => {}
    }
}
