use crate::cmd::r#async as async_cmd;
use crate::engines::functional::{build_metrics, resolve_exit_code};

pub async fn execute(workspace: &str) -> (i32, serde_json::Value) {
    let check = async_cmd("mvn").arg("--version").output().await;
    if !matches!(check, Ok(o) if o.status.success()) {
        let metrics = build_metrics("mvn-test", 0, 0, 0, false, Some("mvn not found on PATH"));
        return (5, metrics);
    }

    let out = async_cmd("mvn")
        .args(["-q", "test"])
        .current_dir(workspace)
        .output()
        .await;

    match out {
        Ok(o) => {
            let combined = format!(
                "{}\n{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            );
            let (passed, failed, skipped) = parse_maven_summary(&combined);
            let no_tests = passed == 0 && failed == 0 && skipped == 0;
            let exit = o.status.code().unwrap_or(1);
            let code = resolve_exit_code(passed, failed, no_tests, exit, false);
            let metrics = build_metrics("mvn-test", passed, failed, skipped, no_tests, None);
            (code, metrics)
        }
        Err(e) => (
            5,
            build_metrics(
                "mvn-test",
                0,
                0,
                0,
                false,
                Some(&format!("Failed to run mvn test: {e}")),
            ),
        ),
    }
}

fn parse_maven_summary(text: &str) -> (u64, u64, u64) {
    for line in text.lines() {
        if line.contains("Tests run:") {
            let tests = extract_num_after(line, "Tests run:");
            let failures = extract_num_after(line, "Failures:");
            let errors = extract_num_after(line, "Errors:");
            let skipped = extract_num_after(line, "Skipped:");
            let failed = failures + errors;
            let passed = tests.saturating_sub(failed).saturating_sub(skipped);
            return (passed, failed, skipped);
        }
    }
    (0, 0, 0)
}

fn extract_num_after(line: &str, key: &str) -> u64 {
    let idx = match line.find(key) {
        Some(i) => i + key.len(),
        None => return 0,
    };
    let rest = line[idx..].trim();
    rest.split(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}
