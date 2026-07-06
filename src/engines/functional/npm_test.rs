use std::path::Path;

use crate::config::load_settings;
use crate::engines::functional::{build_metrics, resolve_exit_code};
use crate::pkgmgr::PackageManager;
use crate::tools;

pub async fn execute(workspace: &str) -> (i32, serde_json::Value) {
    let settings = load_settings();
    let pm = PackageManager::resolve(workspace, &settings.package_manager);
    let root = Path::new(workspace);

    if let Some(pkg) = read_package_json(root) {
        if has_dep(&pkg, "vitest") {
            return run_vitest(workspace, &pm).await;
        }
        if has_dep(&pkg, "jest") || has_dep(&pkg, "@jest/core") {
            return run_jest(workspace, &pm).await;
        }
    }

    run_npm_test(workspace, &pm).await
}

async fn run_vitest(workspace: &str, pm: &PackageManager) -> (i32, serde_json::Value) {
    let report_path = tools::playhouse_dir(workspace).join("reports").join("vitest.json");
    let _ = std::fs::create_dir_all(report_path.parent().unwrap());

    let out = pm
        .exec(
            Path::new(workspace),
            "vitest",
            &[
                "run",
                "--reporter=json",
                &format!("--outputFile={}", report_path.display()),
            ],
        )
        .await;

    match out {
        Ok(o) => parse_vitest_report(&report_path, o.status.code().unwrap_or(1)).await,
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
    let skipped = v["numPendingTests"].as_u64().unwrap_or(0);
    let no_tests = passed == 0 && failed == 0 && skipped == 0;
    let code = resolve_exit_code(passed, failed, no_tests, exit, false);
    let metrics = build_metrics("npm-test", passed, failed, skipped, no_tests, None);
    (code, metrics)
}

async fn run_jest(workspace: &str, pm: &PackageManager) -> (i32, serde_json::Value) {
    let report_path = tools::playhouse_dir(workspace).join("reports").join("jest.json");
    let _ = std::fs::create_dir_all(report_path.parent().unwrap());

    let out = pm
        .exec(
            Path::new(workspace),
            "jest",
            &[
                "--json",
                &format!("--outputFile={}", report_path.display()),
                "--passWithNoTests",
            ],
        )
        .await;

    match out {
        Ok(o) => parse_jest_report(&report_path, o.status.code().unwrap_or(1)).await,
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
    let skipped = v["numPendingTests"].as_u64().unwrap_or(0);
    let no_tests = passed == 0 && failed == 0 && skipped == 0;
    let code = resolve_exit_code(passed, failed, no_tests, exit, false);
    let metrics = build_metrics("npm-test", passed, failed, skipped, no_tests, None);
    (code, metrics)
}

async fn run_npm_test(workspace: &str, pm: &PackageManager) -> (i32, serde_json::Value) {
    match pm.run_test_script(Path::new(workspace)).await {
        Ok(o) => {
            let exit = o.status.code().unwrap_or(1);
            if exit == 0 {
                let metrics = build_metrics("npm-test", 1, 0, 0, false, None);
                (0, metrics)
            } else {
                let metrics = build_metrics("npm-test", 0, 1, 0, false, None);
                (1, metrics)
            }
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
