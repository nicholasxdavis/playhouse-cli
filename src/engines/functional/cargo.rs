use crate::cmd::r#async as async_cmd;
use crate::engines::functional::{build_metrics, resolve_exit_code};
use crate::engines::metrics_util::attach_failure_output;

pub async fn execute(workspace: &str, pattern: Option<&str>) -> (i32, serde_json::Value) {
    let check = async_cmd("cargo").arg("--version").output().await;
    if !matches!(check, Ok(o) if o.status.success()) {
        let metrics = build_metrics(
            "cargo-test",
            0,
            0,
            0,
            false,
            Some("cargo not found on PATH"),
        );
        return (5, metrics);
    }

    let out = {
        let mut cmd = async_cmd("cargo");
        cmd.arg("test").arg("--message-format=json");
        if let Some(p) = pattern {
            cmd.arg(p);
        }
        cmd.current_dir(workspace);
        crate::cmd::output_with_timeout(&mut cmd).await
    };

    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let (passed, failed, skipped) = parse_cargo_output(&stdout, &stderr);
            let no_tests = passed == 0 && failed == 0 && skipped == 0;
            let exit = o.status.code().unwrap_or(1);
            let code = resolve_exit_code(passed, failed, no_tests, exit, false);
            let metrics = attach_failure_output(
                build_metrics("cargo-test", passed, failed, skipped, no_tests, None),
                code,
                &stdout,
                &stderr,
            );
            (code, metrics)
        }
        Err(e) => {
            let metrics = build_metrics(
                "cargo-test",
                0,
                0,
                0,
                false,
                Some(&format!("Failed to run cargo test: {e}")),
            );
            (5, metrics)
        }
    }
}

fn parse_cargo_output(stdout: &str, stderr: &str) -> (u64, u64, u64) {
    let combined = format!("{stdout}\n{stderr}");
    let (passed, failed, skipped) = parse_cargo_json_events(&combined);
    if passed + failed + skipped > 0 {
        return (passed, failed, skipped);
    }
    parse_cargo_human_summary(&combined).unwrap_or((0, 0, 0))
}

fn parse_cargo_json_events(text: &str) -> (u64, u64, u64) {
    let mut passed = 0u64;
    let mut failed = 0u64;
    let mut ignored = 0u64;
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('{') {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if v.get("type").and_then(|t| t.as_str()) == Some("test") {
            match v.get("event").and_then(|e| e.as_str()) {
                Some("ok") => passed += 1,
                Some("failed") => failed += 1,
                Some("ignored") => ignored += 1,
                _ => {}
            }
        }
    }
    (passed, failed, ignored)
}

fn parse_cargo_human_summary(text: &str) -> Option<(u64, u64, u64)> {
    for line in text.lines() {
        if line.contains("test result:") {
            let passed = summary_count(line, "passed");
            let failed = summary_count(line, "failed");
            let ignored = summary_count(line, "ignored");
            return Some((passed, failed, ignored));
        }
    }
    None
}

fn summary_count(line: &str, key: &str) -> u64 {
    let needle = format!("{key};");
    let idx = match line.find(&needle) {
        Some(i) => i,
        None => {
            if let Some(i) = line.find(key) {
                i
            } else {
                return 0;
            }
        }
    };
    let before = &line[..idx];
    before
        .split_whitespace()
        .last()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}
