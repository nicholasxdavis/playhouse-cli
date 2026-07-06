use std::path::Path;

use crate::cmd::{r#async as async_cmd, sync};
use crate::engines::functional::{apply_headless_env, build_metrics, resolve_exit_code};
use crate::engines::metrics_util::attach_failure_output;

pub async fn execute(workspace: &str) -> (i32, serde_json::Value) {
    let gradle = resolve_gradle(workspace);
    let check = sync(&gradle).arg("--version").output();
    if !matches!(check, Ok(o) if o.status.success()) {
        let metrics = build_metrics(
            "gradle-test",
            0,
            0,
            0,
            false,
            Some("gradle/gradlew not found"),
        );
        return (5, metrics);
    }

    let out = {
        let mut cmd = async_cmd(&gradle);
        cmd.args(["test", "--console=plain"]).current_dir(workspace);
        apply_headless_env(&mut cmd);
        cmd.output().await
    };

    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let combined = format!("{stdout}\n{stderr}");
            let (passed, failed, skipped) = parse_gradle_summary(&combined);
            let no_tests = passed == 0 && failed == 0 && skipped == 0;
            let exit = o.status.code().unwrap_or(1);
            let code = resolve_exit_code(passed, failed, no_tests, exit, false);
            let metrics = attach_failure_output(
                build_metrics("gradle-test", passed, failed, skipped, no_tests, None),
                code,
                &stdout,
                &stderr,
            );
            (code, metrics)
        }
        Err(e) => (
            5,
            build_metrics(
                "gradle-test",
                0,
                0,
                0,
                false,
                Some(&format!("Failed to run gradle test: {e}")),
            ),
        ),
    }
}

fn resolve_gradle(workspace: &str) -> String {
    let root = Path::new(workspace);
    #[cfg(windows)]
    {
        if root.join("gradlew.bat").is_file() {
            return root.join("gradlew.bat").to_string_lossy().into_owned();
        }
    }
    #[cfg(not(windows))]
    {
        if root.join("gradlew").is_file() {
            return root.join("gradlew").to_string_lossy().into_owned();
        }
    }
    "gradle".into()
}

fn parse_gradle_summary(text: &str) -> (u64, u64, u64) {
    let mut passed = 0u64;
    let mut failed = 0u64;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("tests completed") {
            let n = rest
                .split_whitespace()
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            passed = n;
        }
        if line.contains("tests completed") && line.contains("failed") {
            if let Some(idx) = line.find("failed") {
                let rest = &line[..idx];
                if let Some(n) = rest
                    .split_whitespace()
                    .rev()
                    .find(|s| s.chars().all(|c| c.is_ascii_digit()))
                {
                    failed = n.parse().unwrap_or(0);
                    passed = passed.saturating_sub(failed);
                }
            }
        }
    }
    (passed, failed, 0)
}
