use std::path::Path;

use crate::config::load_settings;
use crate::engines::metrics_util::{error_metrics, finalize_metrics};
use crate::install;
use crate::pkgmgr::PackageManager;
use crate::report;
use crate::tools;
use serde_json::json;

pub async fn execute(workspace: &str, pattern: Option<&str>) -> (i32, serde_json::Value) {
    let settings = load_settings();
    if settings.auto_install_tools {
        if let Err(e) = install::ensure_playwright(workspace, true).await {
            let metrics = error_metrics(
                "playwright",
                5,
                &e,
                json!({ "fix": "playhouse install --full" }),
            );
            let _ = report::save_engine_report(workspace, "playwright", &metrics);
            return (5, metrics);
        }
    }

    if !tools::has_playwright(workspace) {
        let metrics = error_metrics(
            "playwright",
            5,
            "@playwright/test not installed",
            json!({ "fix": "playhouse install --full" }),
        );
        let _ = report::save_engine_report(workspace, "playwright", &metrics);
        return (5, metrics);
    }

    let ctx = tools::resolve_node_tool_context(workspace);
    let pm = PackageManager::resolve(workspace, &settings.package_manager);
    let cwd = Path::new(&ctx.cwd);
    let prefix = ctx.npm_prefix.as_deref();

    let check = pm.exec(cwd, "playwright", &["--version"]).await;
    if !matches!(check, Ok(o) if o.status.success()) {
        let metrics = error_metrics(
            "playwright",
            5,
            "@playwright/test not runnable",
            json!({
                "source": ctx.source,
                "fix": "playhouse install --full",
            }),
        );
        let _ = report::save_engine_report(workspace, "playwright", &metrics);
        return (5, metrics);
    }

    let mut args = vec!["test", "--reporter=json"];
    if let Some(p) = pattern {
        args.push(p);
    }

    let result = pm
        .exec_with_bin_path(cwd, "playwright", &args, prefix)
        .await;

    match result {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let tool_exit = out.status.code().unwrap_or(1);

            if let Ok(report_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                let stats = &report_json["stats"];
                let expected = stats["expected"].as_u64().unwrap_or(0);
                let unexpected = stats["unexpected"].as_u64().unwrap_or(0);
                let skipped = stats["skipped"].as_u64().unwrap_or(0);
                let flaky = stats["flaky"].as_u64().unwrap_or(0);
                let no_tests = expected == 0 && unexpected == 0 && skipped == 0 && flaky == 0;
                let passed = !no_tests && tool_exit == 0 && unexpected == 0;
                let code = if no_tests {
                    1
                } else if tool_exit == 0 && unexpected == 0 {
                    0
                } else {
                    1
                };
                let mut metrics = finalize_metrics(
                    code,
                    if tool_exit != code {
                        Some(tool_exit)
                    } else {
                        None
                    },
                    json!({
                        "engine": "playwright",
                        "passed": passed,
                        "noTests": no_tests,
                        "source": ctx.source,
                        "stats": {
                            "expected": expected,
                            "unexpected": unexpected,
                            "skipped": skipped,
                            "flaky": flaky,
                            "duration": stats["duration"],
                        },
                    }),
                );
                if code != 0 {
                    if let Some(msg) = extract_playwright_errors(&report_json) {
                        metrics["failureOutput"] = json!(msg);
                    } else {
                        metrics = crate::engines::metrics_util::attach_failure_output(
                            metrics, code, &stdout, &stderr,
                        );
                    }
                }
                let _ = report::save_engine_report(workspace, "playwright", &metrics);
                (code, metrics)
            } else {
                let code = 1;
                let metrics = crate::engines::metrics_util::attach_failure_output(
                    finalize_metrics(
                        code,
                        Some(tool_exit),
                        json!({
                            "engine": "playwright",
                            "passed": false,
                            "noTests": false,
                            "source": ctx.source,
                            "parseError": true,
                            "error": "Playwright JSON report not parsed",
                        }),
                    ),
                    code,
                    &stdout,
                    &stderr,
                );
                let _ = report::save_engine_report(workspace, "playwright", &metrics);
                (code, metrics)
            }
        }
        Err(e) => {
            let metrics = error_metrics(
                "playwright",
                5,
                &format!("Failed to run playwright: {e}"),
                json!({ "source": ctx.source }),
            );
            let _ = report::save_engine_report(workspace, "playwright", &metrics);
            (5, metrics)
        }
    }
}

pub async fn run(workspace: &str, pattern: Option<&str>, json: bool, quiet: bool) -> i32 {
    let (code, metrics) = execute(workspace, pattern).await;

    if !quiet {
        if json {
            println!("{}", serde_json::to_string_pretty(&metrics).unwrap_or_default());
        } else if let Some(err) = metrics.get("error").and_then(|e| e.as_str()) {
            eprintln!("[x] {err}");
        } else if metrics.get("noTests").and_then(|v| v.as_bool()).unwrap_or(false) {
            eprintln!("[x] Playwright: No tests ran. Add tests or run `playhouse test init`");
        } else if let Some(stats) = metrics.get("stats") {
            let expected = stats["expected"].as_u64().unwrap_or(0);
            let unexpected = stats["unexpected"].as_u64().unwrap_or(0);
            let skipped = stats["skipped"].as_u64().unwrap_or(0);
            let flaky = stats["flaky"].as_u64().unwrap_or(0);
            let duration = stats["duration"].as_f64().unwrap_or(0.0);

            println!("Playwright Test Results:");
            println!("  Passed:  {expected}");
            println!("  Failed:  {unexpected}");
            println!("  Skipped: {skipped}");
            println!("  Flaky:   {flaky}");
            println!("  Duration: {:.1}s", duration / 1000.0);

            if code == 0 {
                println!("[*] All tests passed");
            } else {
                println!("[x] Test failures detected");
            }
        }
    }
    code
}

fn extract_playwright_errors(report: &serde_json::Value) -> Option<String> {
    let mut lines = Vec::new();
    if let Some(suites) = report.get("suites").and_then(|s| s.as_array()) {
        collect_playwright_suite_errors(suites, &mut lines);
    }
    if lines.is_empty() {
        return None;
    }
    let joined = lines.join("\n");
    if joined.len() > 8192 {
        Some(joined[joined.len() - 8192..].to_string())
    } else {
        Some(joined)
    }
}

fn collect_playwright_suite_errors(suites: &[serde_json::Value], lines: &mut Vec<String>) {
    for suite in suites {
        if let Some(specs) = suite.get("specs").and_then(|s| s.as_array()) {
            for spec in specs {
                if let Some(tests) = spec.get("tests").and_then(|t| t.as_array()) {
                    for test in tests {
                        let title = test
                            .get("title")
                            .and_then(|t| t.as_str())
                            .unwrap_or("test");
                        if let Some(results) = test.get("results").and_then(|r| r.as_array()) {
                            for result in results {
                                if result.get("status").and_then(|s| s.as_str()) == Some("failed")
                                {
                                    if let Some(msg) = result
                                        .get("error")
                                        .and_then(|e| e.get("message"))
                                        .and_then(|m| m.as_str())
                                    {
                                        lines.push(format!("{title}: {msg}"));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(nested) = suite.get("suites").and_then(|s| s.as_array()) {
            collect_playwright_suite_errors(nested, lines);
        }
    }
}
