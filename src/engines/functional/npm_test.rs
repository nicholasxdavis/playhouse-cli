use std::path::Path;

use crate::config::load_settings;
use crate::engines::functional::{build_metrics, resolve_exit_code};
use crate::pkgmgr::PackageManager;
use crate::tools;

pub async fn execute(workspace: &str, pattern: Option<&str>) -> (i32, serde_json::Value) {
    let settings = load_settings();
    let pm = PackageManager::resolve(workspace, &settings.package_manager);
    let root = Path::new(workspace);

    if let Some(pkg) = read_package_json(root) {
        if has_dep(&pkg, "vitest") {
            return run_vitest(workspace, &pm, pattern).await;
        }
        if has_dep(&pkg, "jest") || has_dep(&pkg, "@jest/core") {
            return run_jest(workspace, &pm, pattern).await;
        }
    }

    run_npm_test(workspace, &pm, pattern).await
}

async fn run_vitest(
    workspace: &str,
    pm: &PackageManager,
    pattern: Option<&str>,
) -> (i32, serde_json::Value) {
    let report_path = tools::playhouse_dir(workspace).join("reports").join("vitest.json");
    let _ = std::fs::create_dir_all(report_path.parent().unwrap());

    let output_file = format!("--outputFile={}", report_path.display());
    let mut args = vec!["run", "--reporter=json", &output_file];
    if let Some(p) = pattern {
        args.push(p);
    }

    let out = pm
        .exec(Path::new(workspace), "vitest", &args)
        .await;

    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let (code, metrics) =
                parse_vitest_report(&report_path, o.status.code().unwrap_or(1)).await;
            (
                code,
                crate::engines::metrics_util::attach_failure_output(
                    metrics, code, &stdout, &stderr,
                ),
            )
        }
        Err(e) => (
            5,
            build_metrics("npm-test", 0, 0, 0, false, Some(&e)),
        ),
    }
}

async fn parse_vitest_report(path: &Path, exit: i32) -> (i32, serde_json::Value) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            let metrics = build_metrics("npm-test", 0, 0, 0, true, None);
            return (1, metrics);
        }
    };
    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            let metrics = build_metrics("npm-test", 0, 0, 0, false, Some("failed to parse vitest JSON"));
            return (5, metrics);
        }
    };
    let passed = v["numPassedTests"].as_u64().unwrap_or(0);
    let failed = v["numFailedTests"].as_u64().unwrap_or(0);
    let skipped = 0;
    let no_tests = passed == 0 && failed == 0;
    let code = resolve_exit_code(passed, failed, no_tests, exit, false);
    let metrics = build_metrics("npm-test", passed, failed, skipped, no_tests, None);
    (code, metrics)
}

async fn run_jest(
    workspace: &str,
    pm: &PackageManager,
    pattern: Option<&str>,
) -> (i32, serde_json::Value) {
    let report_path = tools::playhouse_dir(workspace).join("reports").join("jest.json");
    let _ = std::fs::create_dir_all(report_path.parent().unwrap());

    let output_file = format!("--outputFile={}", report_path.display());
    let mut args = vec!["--json", &output_file, "--passWithNoTests", "--ci", "--watchAll=false"];
    if let Some(p) = pattern {
        args.push(p);
    }

    let out = pm.exec(Path::new(workspace), "jest", &args).await;

    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let (code, metrics) =
                parse_jest_report(&report_path, o.status.code().unwrap_or(1)).await;
            (
                code,
                crate::engines::metrics_util::attach_failure_output(
                    metrics, code, &stdout, &stderr,
                ),
            )
        }
        Err(e) => (
            5,
            build_metrics("npm-test", 0, 0, 0, false, Some(&e)),
        ),
    }
}

async fn parse_jest_report(path: &Path, exit: i32) -> (i32, serde_json::Value) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            let metrics = build_metrics("npm-test", 0, 0, 0, true, None);
            return (1, metrics);
        }
    };
    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            let metrics = build_metrics("npm-test", 0, 0, 0, false, Some("failed to parse jest JSON"));
            return (5, metrics);
        }
    };
    let passed = v["numPassedTests"].as_u64().unwrap_or(0);
    let failed = v["numFailedTests"].as_u64().unwrap_or(0);
    let skipped = 0;
    let no_tests = passed == 0 && failed == 0;
    let code = resolve_exit_code(passed, failed, no_tests, exit, false);
    let metrics = build_metrics("npm-test", passed, failed, skipped, no_tests, None);
    (code, metrics)
}

async fn run_npm_test(
    workspace: &str,
    pm: &PackageManager,
    pattern: Option<&str>,
) -> (i32, serde_json::Value) {
    let cwd = Path::new(workspace);
    let out = if let Some(p) = pattern {
        pm.run_test_script_with(cwd, &[p]).await
    } else {
        pm.run_test_script(cwd).await
    };
    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let exit = o.status.code().unwrap_or(1);
            let code = if exit == 0 { 0 } else { 1 };
            let metrics = build_metrics("npm-test", 0, 0, 0, false, None);
            (
                code,
                crate::engines::metrics_util::attach_failure_output(
                    metrics, code, &stdout, &stderr,
                ),
            )
        }
        Err(e) => (
            5,
            build_metrics("npm-test", 0, 0, 0, false, Some(&e)),
        ),
    }
}

fn read_package_json(root: &Path) -> Option<serde_json::Value> {
    let content = std::fs::read_to_string(root.join("package.json")).ok()?;
    serde_json::from_str(&content).ok()
}

fn has_dep(pkg: &serde_json::Value, name: &str) -> bool {
    ["devDependencies", "dependencies"]
        .iter()
        .filter_map(|k| pkg.get(*k).and_then(|d| d.as_object()))
        .any(|deps| deps.contains_key(name))
}
