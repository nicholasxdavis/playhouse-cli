use crate::cmd::r#async as async_cmd;
use crate::engines::functional::{build_metrics, resolve_exit_code};

pub async fn execute(workspace: &str) -> (i32, serde_json::Value) {
    let check = async_cmd("go").arg("version").output().await;
    if !matches!(check, Ok(o) if o.status.success()) {
        let metrics = build_metrics("go-test", 0, 0, 0, false, Some("go not found on PATH"));
        return (5, metrics);
    }

    let out = async_cmd("go")
        .args(["test", "-json", "./..."])
        .current_dir(workspace)
        .output()
        .await;

    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let (passed, failed, skipped) = parse_go_json(&stdout);
            let no_tests = passed == 0 && failed == 0 && skipped == 0;
            let exit = o.status.code().unwrap_or(1);
            let code = resolve_exit_code(passed, failed, no_tests, exit, false);
            let metrics = build_metrics("go-test", passed, failed, skipped, no_tests, None);
            (code, metrics)
        }
        Err(e) => {
            let metrics = build_metrics(
                "go-test",
                0,
                0,
                0,
                false,
                Some(&format!("Failed to run go test: {e}")),
            );
            (5, metrics)
        }
    }
}

fn parse_go_json(stdout: &str) -> (u64, u64, u64) {
    let mut passed = 0u64;
    let mut failed = 0u64;
    let mut skipped = 0u64;
    for line in stdout.lines() {
        let line = line.trim();
        if !line.starts_with('{') {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        match v.get("Action").and_then(|a| a.as_str()) {
            Some("pass") => passed += 1,
            Some("fail") => failed += 1,
            Some("skip") => skipped += 1,
            _ => {}
        }
    }
    (passed, failed, skipped)
}
