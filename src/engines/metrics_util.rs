use serde_json::{json, Value};

const FAILURE_OUTPUT_MAX_BYTES: usize = 8192;
const FAILURE_OUTPUT_LINES: usize = 50;

/// Last lines of stderr (or stdout if stderr empty) for failed runs.
pub fn tail_output(stdout: &str, stderr: &str) -> Option<String> {
    let prefer = if !stderr.trim().is_empty() {
        stderr
    } else if !stdout.trim().is_empty() {
        stdout
    } else {
        return None;
    };
    let lines: Vec<&str> = prefer.lines().collect();
    let start = lines.len().saturating_sub(FAILURE_OUTPUT_LINES);
    let tail = lines[start..].join("\n");
    if tail.len() > FAILURE_OUTPUT_MAX_BYTES {
        Some(tail[tail.len() - FAILURE_OUTPUT_MAX_BYTES..].to_string())
    } else {
        Some(tail)
    }
}

pub fn attach_failure_output(mut metrics: Value, code: i32, stdout: &str, stderr: &str) -> Value {
    if code != 0 {
        if let Some(tail) = tail_output(stdout, stderr) {
            metrics["failureOutput"] = json!(tail);
        }
    }
    metrics
}

/// Attach Playhouse exit code to engine metrics. Raw tool exit goes in `toolExitCode` when it differs.
pub fn finalize_metrics(playhouse_code: i32, tool_exit: Option<i32>, mut metrics: Value) -> Value {
    if let Some(t) = tool_exit {
        if t != playhouse_code {
            metrics["toolExitCode"] = json!(t);
        }
    }
    metrics["exitCode"] = json!(playhouse_code);
    metrics
}

pub fn error_metrics(engine: &str, playhouse_code: i32, error: &str, extra: Value) -> Value {
    let mut metrics = json!({
        "engine": engine,
        "error": error,
        "passed": false,
        "scanComplete": false,
    });
    if let Value::Object(ref mut map) = metrics {
        if let Value::Object(ext) = extra {
            for (k, v) in ext {
                map.insert(k, v);
            }
        }
    }
    finalize_metrics(playhouse_code, None, metrics)
}
