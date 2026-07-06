use std::path::Path;

use crate::cmd::r#async as async_cmd;
use crate::engines::functional::{build_metrics, resolve_exit_code};
use crate::engines::metrics_util::attach_failure_output;
use crate::tools;

pub async fn execute(workspace: &str, pattern: Option<&str>) -> (i32, serde_json::Value) {
    let program = if cfg!(windows) { "pytest.exe" } else { "pytest" };
    let check = async_cmd(program).arg("--version").output().await;
    let check_ok = matches!(&check, Ok(o) if o.status.success());
    if !check_ok {
        let fallback = async_cmd(if cfg!(windows) { "python.exe" } else { "python" })
            .args(["-m", "pytest", "--version"])
            .output()
            .await;
        if !matches!(fallback, Ok(o) if o.status.success()) {
            let metrics = build_metrics(
                "pytest",
                0,
                0,
                0,
                false,
                Some("pytest not found - pip install pytest"),
            );
            return (5, metrics);
        }
    }

    let report_dir = tools::playhouse_dir(workspace).join("reports");
    let _ = std::fs::create_dir_all(&report_dir);
    let junit_path = report_dir.join("pytest-junit.xml");

    let junit_arg = format!("--junitxml={}", junit_path.display());
    let mut pytest_args = vec!["-m", "pytest", "--tb=short", "-q", &junit_arg];
    if let Some(p) = pattern {
        pytest_args.push(p);
    }

    let out = async_cmd(if cfg!(windows) { "python.exe" } else { "python" })
        .args(&pytest_args)
        .current_dir(workspace)
        .output()
        .await;

    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let (passed, failed, skipped) = parse_junit(&junit_path);
            let no_tests = passed == 0 && failed == 0 && skipped == 0;
            let exit = o.status.code().unwrap_or(1);
            let code = resolve_exit_code(passed, failed, no_tests, exit, false);
            let metrics = attach_failure_output(
                build_metrics("pytest", passed, failed, skipped, no_tests, None),
                code,
                &stdout,
                &stderr,
            );
            (code, metrics)
        }
        Err(e) => {
            let metrics = build_metrics(
                "pytest",
                0,
                0,
                0,
                false,
                Some(&format!("Failed to run pytest: {e}")),
            );
            (5, metrics)
        }
    }
}

fn parse_junit(path: &Path) -> (u64, u64, u64) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (0, 0, 0),
    };
    parse_junit_attrs(&content)
}

fn parse_junit_attrs(content: &str) -> (u64, u64, u64) {
    let mut tests = 0u64;
    let mut failures = 0u64;
    let mut skipped = 0u64;
    for tag in ["testsuites", "testsuite"] {
        if let Some(attrs) = extract_tag_attrs(content, tag) {
            tests = tests.max(parse_attr_u64(&attrs, "tests"));
            failures = failures.max(parse_attr_u64(&attrs, "failures") + parse_attr_u64(&attrs, "errors"));
            skipped = skipped.max(parse_attr_u64(&attrs, "skipped"));
        }
    }
    let passed = tests.saturating_sub(failures).saturating_sub(skipped);
    (passed, failures, skipped)
}

fn extract_tag_attrs(content: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let start = content.find(&open)?;
    let rest = &content[start..];
    let end = rest.find('>')?;
    Some(rest[..end].to_string())
}

fn parse_attr_u64(attrs: &str, name: &str) -> u64 {
    let needle = format!("{name}=\"");
    let start = match attrs.find(&needle) {
        Some(i) => i + needle.len(),
        None => return 0,
    };
    let rest = &attrs[start..];
    let end = rest.find('"').unwrap_or(rest.len());
    rest[..end].parse().unwrap_or(0)
}
