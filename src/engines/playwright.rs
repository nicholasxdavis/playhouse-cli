use std::path::Path;

use crate::config::load_settings;
use crate::install;
use crate::pkgmgr::PackageManager;
use crate::report;
use crate::tools;

pub async fn execute(workspace: &str, pattern: Option<&str>) -> (i32, serde_json::Value) {
    let settings = load_settings();
    if settings.auto_install_tools {
        if let Err(e) = install::ensure_playwright(workspace, true).await {
            let metrics = serde_json::json!({
                "engine": "playwright",
                "error": e,
                "fix": "playhouse install --full",
                "passed": false,
            });
            let _ = report::save_engine_report(workspace, "playwright", &metrics);
            return (5, metrics);
        }
    }

    if !tools::has_playwright(workspace) {
        let metrics = serde_json::json!({
            "engine": "playwright",
            "error": "@playwright/test not installed",
            "fix": "playhouse install --full",
            "passed": false,
        });
        let _ = report::save_engine_report(workspace, "playwright", &metrics);
        return (5, metrics);
    }

    let ctx = tools::resolve_node_tool_context(workspace);
    let pm = PackageManager::resolve(workspace, &settings.package_manager);
    let cwd = Path::new(&ctx.cwd);
    let prefix = ctx.npm_prefix.as_deref();

    let check = pm.exec(cwd, "playwright", &["--version"]).await;
    if !matches!(check, Ok(o) if o.status.success()) {
        let metrics = serde_json::json!({
            "engine": "playwright",
            "error": "@playwright/test not runnable",
            "source": ctx.source,
            "fix": "playhouse install --full",
            "passed": false,
        });
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
            let exit = out.status.code().unwrap_or(1);

            if let Ok(report_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                let stats = &report_json["stats"];
                let expected = stats["expected"].as_u64().unwrap_or(0);
                let unexpected = stats["unexpected"].as_u64().unwrap_or(0);
                let skipped = stats["skipped"].as_u64().unwrap_or(0);
                let flaky = stats["flaky"].as_u64().unwrap_or(0);
                let no_tests = expected == 0 && unexpected == 0 && skipped == 0 && flaky == 0;
                let passed = !no_tests && exit == 0 && unexpected == 0;
                let code = if no_tests {
                    1
                } else if exit == 0 && unexpected == 0 {
                    0
                } else {
                    1
                };
                let metrics = serde_json::json!({
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
                });
                let _ = report::save_engine_report(workspace, "playwright", &metrics);
                (code, metrics)
            } else {
                let code = if exit == 0 { 0 } else { 1 };
                let metrics = serde_json::json!({
                    "engine": "playwright",
                    "passed": exit == 0,
                    "noTests": false,
                    "source": ctx.source,
                    "parseError": stdout.is_empty(),
                    "exitCode": exit,
                });
                let _ = report::save_engine_report(workspace, "playwright", &metrics);
                (code, metrics)
            }
        }
        Err(e) => {
            let metrics = serde_json::json!({
                "engine": "playwright",
                "error": format!("Failed to run playwright: {e}"),
                "source": ctx.source,
                "passed": false,
            });
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
            eprintln!("[x] Playwright: No tests ran — add tests or run `playhouse test init`");
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
