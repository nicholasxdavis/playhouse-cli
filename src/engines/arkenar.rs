use std::path::Path;

use crate::cmd::r#async as async_cmd;
use crate::config::load_settings;
use crate::engines::metrics_util::{attach_failure_output, error_metrics, finalize_metrics};
use crate::install;
use crate::report;
use crate::tools;
use serde_json::{json, Value};

pub async fn execute(url: &str, workspace: &str) -> (i32, serde_json::Value) {
    if load_settings().auto_install_tools {
        if let Err(e) = install::ensure_arkenar(true).await {
            let metrics = error_metrics("arkenar", 5, &e, json!({ "fix": "playhouse install" }));
            let _ = report::save_engine_report(workspace, "arkenar", &metrics);
            return (5, metrics);
        }
    }

    let program = tools::arkenar_program();
    let check = async_cmd(&program).arg("--help").output().await;
    if !matches!(check, Ok(o) if o.status.success()) {
        let metrics = error_metrics(
            "arkenar",
            5,
            "arkenar not installed",
            json!({ "fix": "playhouse install" }),
        );
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

    if let Some(headers) = crate::workspace::resolved_audit_headers(workspace) {
        args.extend(crate::workspace::arkenar_header_args(&headers));
    }

    args.push(url.to_string());

    if settings.arkenar_param_fuzz {
        args.push("--enable-param-fuzz".into());
    }
    if settings.arkenar_js_analysis {
        args.push("--enable-js-analysis".into());
    }

    let result = {
        let mut cmd = async_cmd(&program);
        cmd.args(&args);
        crate::cmd::output_with_timeout(&mut cmd).await
    };

    match result {
        Ok(out) => {
            let tool_exit = out.status.code().unwrap_or(1);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let (findings, parse_detail) = parse_report_detailed(&output_path, &stdout);

            if findings.is_none() {
                let code = if tool_exit != 0 { tool_exit } else { 5 };
                let failure_mode = arkenar_failure_mode(&output_path, tool_exit, &parse_detail);
                let mut metrics = finalize_metrics(
                    code,
                    if tool_exit != code {
                        Some(tool_exit)
                    } else {
                        None
                    },
                    json!({
                        "engine": "arkenar",
                        "passed": false,
                        "scanComplete": false,
                        "reportParseError": true,
                        "failureMode": failure_mode,
                        "target": url,
                        "reportPath": output_path,
                        "reportFileExists": output_path.is_file(),
                        "reportFileBytes": std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0),
                        "parseDetail": parse_detail,
                        "error": arkenar_error_message(failure_mode, &output_path),
                    }),
                );
                metrics = attach_failure_output(metrics, code, &stdout, &stderr);
                let _ = report::save_engine_report(workspace, "arkenar", &metrics);
                return (code, metrics);
            }

            let data = findings.unwrap();
            if !report_file_valid(&output_path) && report_is_empty(&data) {
                let code = if tool_exit != 0 { tool_exit } else { 5 };
                let mut metrics = finalize_metrics(
                    code,
                    Some(tool_exit),
                    json!({
                        "engine": "arkenar",
                        "passed": false,
                        "scanComplete": false,
                        "reportParseError": true,
                        "failureMode": "empty_report",
                        "target": url,
                        "reportPath": output_path,
                        "error": "arkenar produced no usable report",
                    }),
                );
                metrics = attach_failure_output(metrics, code, &stdout, &stderr);
                let _ = report::save_engine_report(workspace, "arkenar", &metrics);
                return (code, metrics);
            }

            let (high, medium, low, total) = summarize_findings(&data);
            if tool_exit != 0 && total == 0 {
                let code = if tool_exit != 0 { tool_exit } else { 5 };
                let mut metrics = finalize_metrics(
                    code,
                    Some(tool_exit),
                    json!({
                        "engine": "arkenar",
                        "passed": false,
                        "scanComplete": false,
                        "failureMode": "target_error",
                        "target": url,
                        "reportPath": output_path,
                        "error": "arkenar exited with error and no findings",
                    }),
                );
                metrics = attach_failure_output(metrics, code, &stdout, &stderr);
                let _ = report::save_engine_report(workspace, "arkenar", &metrics);
                return (code, metrics);
            }

            let threshold_fail = high > 0 || medium > 0;
            let passed = !threshold_fail && tool_exit == 0;
            let code = if threshold_fail {
                3
            } else if tool_exit != 0 {
                1
            } else {
                0
            };
            let metrics = finalize_metrics(
                code,
                if tool_exit != code {
                    Some(tool_exit)
                } else {
                    None
                },
                json!({
                    "engine": "arkenar",
                    "passed": passed,
                    "scanComplete": true,
                    "target": url,
                    "reportPath": output_path,
                    "summary": {
                        "high": high,
                        "medium": medium,
                        "low": low,
                        "total": total,
                    },
                    "raw": data,
                }),
            );
            let _ = report::save_engine_report(workspace, "arkenar", &metrics);
            (code, metrics)
        }
        Err(e) => {
            let metrics = attach_failure_output(
                error_metrics(
                    "arkenar",
                    5,
                    &format!("Failed to run arkenar: {e}"),
                    json!({ "failureMode": "spawn_error" }),
                ),
                5,
                "",
                &e.to_string(),
            );
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

fn report_file_valid(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.len() > 2)
        .unwrap_or(false)
}

fn report_is_empty(data: &Value) -> bool {
    match data {
        Value::Object(m) => m.is_empty(),
        Value::Array(a) => a.is_empty(),
        Value::Null => true,
        _ => false,
    }
}

fn parse_report(path: &Path) -> Option<Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn parse_report_detailed(path: &Path, stdout: &str) -> (Option<Value>, String) {
    if let Some(v) = parse_report(path) {
        return (Some(v), "parsed report file".into());
    }
    if !path.is_file() {
        if let Some(v) = parse_stdout_json(stdout) {
            return (Some(v), "parsed stdout (report file missing)".into());
        }
        return (None, "report file missing".into());
    }
    if let Ok(content) = std::fs::read_to_string(path) {
        if content.trim().is_empty() {
            return (None, "report file empty".into());
        }
        return (
            None,
            format!(
                "report parse error: {}",
                content.chars().take(80).collect::<String>()
            ),
        );
    }
    if let Some(v) = parse_stdout_json(stdout) {
        return (Some(v), "parsed stdout fallback".into());
    }
    (None, "failed to parse report file or stdout".into())
}

fn arkenar_failure_mode(path: &Path, tool_exit: i32, detail: &str) -> &'static str {
    if !path.is_file() {
        if tool_exit != 0 {
            return "target_error";
        }
        return "missing_report";
    }
    if detail.contains("empty") {
        return "empty_report";
    }
    "parse_error"
}

fn arkenar_error_message(mode: &str, path: &Path) -> String {
    match mode {
        "missing_report" => "arkenar report file missing".into(),
        "empty_report" => "arkenar report file empty".into(),
        "target_error" => "arkenar target unreachable or exited with error".into(),
        "parse_error" => format!("failed to parse arkenar report at {}", path.display()),
        _ => "arkenar scan failed".into(),
    }
}

fn parse_stdout_json(stdout: &str) -> Option<Value> {
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

fn summarize_findings(data: &Value) -> (u64, u64, u64, u64) {
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

fn walk_findings(value: &Value, on_severity: &mut dyn FnMut(&str)) {
    match value {
        Value::Array(arr) => {
            for item in arr {
                walk_findings(item, on_severity);
            }
        }
        Value::Object(map) => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_report_detailed_missing_file() {
        let (v, detail) = parse_report_detailed(&PathBuf::from("/nonexistent/arkenar.json"), "");
        assert!(v.is_none());
        assert!(detail.contains("missing"));
    }

    #[test]
    fn failure_mode_missing_report() {
        assert_eq!(
            arkenar_failure_mode(Path::new("/missing.json"), 0, "report file missing"),
            "missing_report"
        );
    }
}
