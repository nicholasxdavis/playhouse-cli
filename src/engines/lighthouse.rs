use std::path::Path;

use crate::cmd::r#async as async_cmd;
use crate::config::load_settings;
use crate::engines::metrics_util::{error_metrics, finalize_metrics};
use crate::install;
use crate::pkgmgr::PackageManager;
use crate::report;
use crate::tools;
use crate::types::LighthouseScores;
use serde::Deserialize;
use serde_json::json;

pub async fn execute(url: &str, workspace: &str, settings: &crate::config::PlayhouseSettings) -> (i32, serde_json::Value) {
    if settings.auto_install_tools {
        if let Err(e) = install::ensure_lighthouse(workspace, true).await {
            let metrics = error_metrics(
                "lighthouse",
                5,
                &e,
                json!({ "fix": "playhouse install --full" }),
            );
            let _ = report::save_engine_report(workspace, "lighthouse", &metrics);
            return (5, metrics);
        }
    }

    let raw = match exec_lighthouse(url, workspace, settings).await {
        Ok(r) => r,
        Err(e) => {
            let metrics = error_metrics("lighthouse", 5, &e, json!({}));
            let _ = report::save_engine_report(workspace, "lighthouse", &metrics);
            return (5, metrics);
        }
    };

    let scores = match parse_scores(&raw) {
        Ok(s) => s,
        Err(e) => {
            let metrics = error_metrics("lighthouse", 5, &e, json!({}));
            let _ = report::save_engine_report(workspace, "lighthouse", &metrics);
            return (5, metrics);
        }
    };

    if !scores.has_any() {
        let metrics = error_metrics(
            "lighthouse",
            5,
            "Lighthouse returned no category scores",
            json!({ "url": url }),
        );
        let _ = report::save_engine_report(workspace, "lighthouse", &metrics);
        return (5, metrics);
    }

    let threshold = settings.lighthouse_threshold;
    let passed = scores.all_pass(threshold);
    let code = if passed { 0 } else { 2 };
    let metrics = finalize_metrics(
        code,
        None,
        json!({
            "engine": "lighthouse",
            "url": url,
            "passed": passed,
            "scanComplete": true,
            "threshold": threshold,
            "scores": {
                "performance": scores.performance,
                "accessibility": scores.accessibility,
                "bestPractices": scores.best_practices,
                "seo": scores.seo,
            },
        }),
    );
    let _ = report::save_engine_report(workspace, "lighthouse", &metrics);
    (code, metrics)
}

pub async fn run(url: &str, workspace: &str, json: bool, quiet: bool) -> i32 {
    let settings = load_settings();
    let (code, metrics) = execute(url, workspace, &settings).await;

    if !quiet {
        if json {
            println!("{}", serde_json::to_string_pretty(&metrics).unwrap_or_default());
        } else if let Some(err) = metrics.get("error").and_then(|e| e.as_str()) {
            eprintln!("[x] Lighthouse error: {err}");
        } else {
            let scores = LighthouseScores {
                performance: metrics["scores"]["performance"].as_f64(),
                accessibility: metrics["scores"]["accessibility"].as_f64(),
                best_practices: metrics["scores"]["bestPractices"].as_f64(),
                seo: metrics["scores"]["seo"].as_f64(),
            };
            print!("{}", report::format_lighthouse_text(url, &scores));
            if metrics["passed"].as_bool().unwrap_or(false) {
                println!("[*] All scores meet minimum threshold");
            } else {
                println!("[x] One or more scores below threshold");
            }
        }
    }
    code
}

#[derive(Deserialize)]
struct LighthouseReport {
    categories: LighthouseCategories,
}

#[derive(Deserialize)]
struct LighthouseCategories {
    performance: Option<CategoryScore>,
    accessibility: Option<CategoryScore>,
    #[serde(rename = "best-practices")]
    best_practices: Option<CategoryScore>,
    seo: Option<CategoryScore>,
}

#[derive(Deserialize)]
struct CategoryScore {
    score: Option<f64>,
}

async fn exec_lighthouse(
    url: &str,
    workspace: &str,
    settings: &crate::config::PlayhouseSettings,
) -> Result<String, String> {
    let pm = PackageManager::resolve(workspace, &settings.package_manager);
    let ctx = tools::resolve_node_tool_context(workspace);
    let mut lh_args: Vec<String> = vec![
        url.into(),
        "--output=json".into(),
        "--quiet".into(),
        "--chrome-flags=--headless".into(),
    ];
    if let Some(headers) = crate::workspace::audit_headers(workspace) {
        if !headers.is_empty() {
            lh_args.push(format!(
                "--extra-headers={}",
                crate::workspace::lighthouse_extra_headers_json(&headers)
            ));
        }
    }
    let lh_refs: Vec<&str> = lh_args.iter().map(String::as_str).collect();
    let cwd = Path::new(&ctx.cwd);
    let prefix = ctx.npm_prefix.as_deref();

    let mut last_error = String::new();

    if tools::has_lighthouse(workspace) {
        match pm
            .exec_with_bin_path(cwd, "lighthouse", &lh_refs, prefix)
            .await
        {
            Ok(out) => {
                if let Some(stdout) = lighthouse_stdout(&out) {
                    return Ok(stdout);
                }
                last_error = lighthouse_failure("lighthouse", &out, last_error);
            }
            Err(e) => {
                last_error = e;
            }
        }
    }

    if let Ok(out) = async_cmd("lighthouse")
        .args(&lh_refs)
        .current_dir(workspace)
        .output()
        .await
    {
        if let Some(stdout) = lighthouse_stdout(&out) {
            return Ok(stdout);
        }
        last_error = lighthouse_failure("lighthouse", &out, last_error);
    }

    Err(if last_error.is_empty() {
        "Lighthouse not found - run: playhouse install --full".to_string()
    } else {
        last_error
    })
}

fn lighthouse_stdout(out: &std::process::Output) -> Option<String> {
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !stdout.is_empty() && stdout.starts_with('{') {
        Some(stdout)
    } else {
        None
    }
}

fn lighthouse_failure(cmd: &str, out: &std::process::Output, prev: String) -> String {
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if !stderr.is_empty() {
        stderr
    } else if !out.status.success() {
        format!("{cmd} exited with code {}", out.status.code().unwrap_or(-1))
    } else {
        prev
    }
}

fn parse_scores(json_str: &str) -> Result<LighthouseScores, String> {
    let report: LighthouseReport =
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse Lighthouse JSON: {e}"))?;

    Ok(LighthouseScores {
        performance: report.categories.performance.and_then(|c| c.score),
        accessibility: report.categories.accessibility.and_then(|c| c.score),
        best_practices: report.categories.best_practices.and_then(|c| c.score),
        seo: report.categories.seo.and_then(|c| c.score),
    })
}
