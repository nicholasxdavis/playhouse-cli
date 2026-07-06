use serde_json::Value;

pub mod cargo;
pub mod go;
pub mod gradle;
pub mod mvn;
pub mod npm_test;
pub mod pytest;

use crate::config::load_settings;
use crate::engines::metrics_util::finalize_metrics;
use crate::install;
use crate::project::{FunctionalRunner, ProjectProfile};
use crate::report;

/// Environment variables injected for headless functional test runs.
pub const HEADLESS_ENV: &[(&str, &str)] = &[
    ("CI", "true"),
    ("WATCH", "false"),
    ("FORCE_COLOR", "0"),
];

/// Apply CI-style env vars so test runners do not enter interactive watch mode.
pub fn apply_headless_env(cmd: &mut tokio::process::Command) {
    for (key, value) in HEADLESS_ENV {
        cmd.env(key, *value);
    }
}

pub async fn execute(
    workspace: &str,
    profile: &ProjectProfile,
    pattern: Option<&str>,
) -> (i32, Value) {
    let settings = load_settings();
    if settings.auto_install_tools {
        if let Err(e) = ensure_runner_tools(workspace, profile).await {
            let metrics = error_metrics(profile.functional_runner.as_str(), &e);
            let _ = report::save_engine_report(workspace, "functional", &metrics);
            return (5, metrics);
        }
    }

    let runner = profile.functional_runner;
    let test_root = crate::workspace::test_root_str(workspace);
    let (code, metrics) = match runner {
        FunctionalRunner::Playwright => {
            let (code, m) = crate::engines::playwright::execute(workspace, pattern).await;
            (code, normalize_playwright(&m, runner))
        }
        FunctionalRunner::CargoTest => cargo::execute(&test_root, pattern).await,
        FunctionalRunner::GoTest => go::execute(&test_root, pattern).await,
        FunctionalRunner::Pytest => pytest::execute(&test_root, pattern).await,
        FunctionalRunner::NpmTest => npm_test::execute(&test_root, pattern).await,
        FunctionalRunner::MvnTest => mvn::execute(&test_root).await,
        FunctionalRunner::GradleTest => gradle::execute(&test_root).await,
        FunctionalRunner::None => {
            let metrics = finalize_metrics(
                1,
                None,
                serde_json::json!({
                    "engine": "functional",
                    "runner": "none",
                    "skipped": true,
                    "passed": false,
                    "noTests": true,
                    "stats": { "passed": 0, "failed": 0, "skipped": 0, "total": 0 },
                }),
            );
            let _ = report::save_engine_report(workspace, "functional", &metrics);
            return (1, metrics);
        }
    };

    let mut metrics = metrics;
    if metrics.get("engine").is_none() {
        metrics["engine"] = Value::String("functional".into());
    }
    if metrics.get("exitCode").is_none() {
        metrics = finalize_metrics(code, None, metrics);
    }
    metrics["headlessEnv"] = serde_json::json!(true);
    if let Some(p) = pattern {
        metrics["testPattern"] = serde_json::json!(p);
    }
    let roots = crate::workspace::resolve_roots(workspace);
    metrics["roots"] = serde_json::json!({
        "workspace": roots.workspace.to_string_lossy(),
        "scan": roots.scan.to_string_lossy(),
        "test": roots.test.to_string_lossy(),
    });
    let _ = report::save_engine_report(workspace, "functional", &metrics);
    (code, metrics)
}

pub async fn run(workspace: &str, pattern: Option<&str>, json: bool, quiet: bool) -> i32 {
    let profile = crate::project::detect(workspace);
    let (code, metrics) = execute(workspace, &profile, pattern).await;

    if !quiet {
        if json {
            println!("{}", serde_json::to_string_pretty(&metrics).unwrap_or_default());
        } else {
            print_human(&metrics, code);
        }
    }
    code
}

async fn ensure_runner_tools(workspace: &str, profile: &ProjectProfile) -> Result<(), String> {
    match profile.functional_runner {
        FunctionalRunner::Playwright => install::ensure_playwright(workspace, true).await.map(|_| ()),
        FunctionalRunner::NpmTest => Ok(()),
        _ => Ok(()),
    }
}

pub fn build_metrics(
    runner: &str,
    passed: u64,
    failed: u64,
    skipped: u64,
    no_tests: bool,
    error: Option<&str>,
) -> Value {
    let total = passed + failed;
    let ok = error.is_none() && !no_tests && failed == 0;
    let mut m = serde_json::json!({
        "engine": "functional",
        "runner": runner,
        "passed": ok,
        "noTests": no_tests,
        "stats": {
            "passed": passed,
            "failed": failed,
            "skipped": skipped,
            "total": total,
        },
    });
    if let Some(err) = error {
        m["error"] = Value::String(err.into());
        m["fix"] = Value::String(fix_hint(runner));
    }
    m
}

pub fn resolve_exit_code(
    _passed: u64,
    failed: u64,
    no_tests: bool,
    process_exit: i32,
    errored: bool,
) -> i32 {
    if errored {
        5
    } else if no_tests || failed > 0 || process_exit != 0 {
        1
    } else {
        0
    }
}

fn error_metrics(runner: &str, err: &str) -> Value {
    build_metrics(runner, 0, 0, 0, false, Some(err))
}

fn normalize_playwright(m: &Value, runner: FunctionalRunner) -> Value {
    let runner = runner.as_str();
    if m.get("error").is_some() {
        let err = m.get("error").and_then(|e| e.as_str()).unwrap_or("playwright error");
        return build_metrics(runner, 0, 0, 0, false, Some(err));
    }
    let no_tests = m.get("noTests").and_then(|v| v.as_bool()).unwrap_or(false);
    let stats = &m["stats"];
    let passed = stats["expected"].as_u64().unwrap_or(0);
    let failed = stats["unexpected"].as_u64().unwrap_or(0);
    let skipped = stats["skipped"].as_u64().unwrap_or(0);
    let mut out = build_metrics(runner, passed, failed, skipped, no_tests, None);
    if let Some(src) = m.get("source") {
        out["source"] = src.clone();
    }
    if let Some(fo) = m.get("failureOutput") {
        out["failureOutput"] = fo.clone();
    }
    out
}

#[cfg(test)]
mod headless_tests {
    use super::*;

    #[test]
    fn headless_env_includes_ci() {
        assert!(HEADLESS_ENV.iter().any(|(k, v)| *k == "CI" && *v == "true"));
        assert!(HEADLESS_ENV.iter().any(|(k, _)| *k == "WATCH"));
    }
}

fn fix_hint(runner: &str) -> String {
    match runner {
        "playwright" => "playhouse install --full".into(),
        "cargo-test" => "install Rust toolchain: https://rustup.rs".into(),
        "go-test" => "install Go: https://go.dev/dl/".into(),
        "pytest" => "pip install pytest".into(),
        "npm-test" => "npm install; npm test".into(),
        "mvn-test" => "install Maven and JDK".into(),
        "gradle-test" => "install Gradle/JDK or add gradlew".into(),
        _ => "playhouse doctor --json".into(),
    }
}

fn print_human(metrics: &Value, code: i32) {
    if let Some(err) = metrics.get("error").and_then(|e| e.as_str()) {
        eprintln!("[x] {err}");
        return;
    }
    if metrics.get("skipped").and_then(|v| v.as_bool()).unwrap_or(false) {
        println!("[-] No functional runner for this project");
        return;
    }
    let runner = metrics["runner"].as_str().unwrap_or("functional");
    if metrics.get("noTests").and_then(|v| v.as_bool()).unwrap_or(false) {
        eprintln!("[x] {runner}: No tests ran");
        return;
    }
    let stats = &metrics["stats"];
    let passed = stats["passed"].as_u64().unwrap_or(0);
    let failed = stats["failed"].as_u64().unwrap_or(0);
    let skipped = stats["skipped"].as_u64().unwrap_or(0);
    println!("Functional ({runner}):");
    println!("  Passed:  {passed}");
    println!("  Failed:  {failed}");
    println!("  Skipped: {skipped}");
    if code == 0 {
        println!("[*] All tests passed");
    } else {
        println!("[x] Test failures detected");
    }
}
