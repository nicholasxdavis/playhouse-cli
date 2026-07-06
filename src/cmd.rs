use std::path::Path;
use std::process::{Command as SyncCommand, Output, Stdio};
use std::time::Duration;

use tokio::process::Command as AsyncCommand;

/// Default wall-clock limit for engine subprocesses (10 minutes).
pub const ENGINE_TIMEOUT_SECS: u64 = 600;

pub fn npm_program() -> &'static str {
    #[cfg(windows)]
    {
        "npm.cmd"
    }
    #[cfg(not(windows))]
    {
        "npm"
    }
}

fn is_direct_program(program: &str) -> bool {
    matches!(program, "npm" | "npx" | "node" | "npm.cmd" | "npx.cmd")
}

#[allow(dead_code)]
pub fn npx_program() -> &'static str {
    #[cfg(windows)]
    {
        "npx.cmd"
    }
    #[cfg(not(windows))]
    {
        "npx"
    }
}

pub fn sync(program: &str) -> SyncCommand {
    if is_direct_executable(program) || is_direct_program(program) {
        let mut cmd = SyncCommand::new(program);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        return cmd;
    }
    let mut cmd = wrap_sync(program);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

pub fn r#async(program: &str) -> AsyncCommand {
    if is_direct_executable(program) || is_direct_program(program) {
        let mut cmd = AsyncCommand::new(program);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        return cmd;
    }
    let mut cmd = wrap_async(program);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

pub async fn output_with_timeout(cmd: &mut AsyncCommand) -> Result<Output, String> {
    let timeout = Duration::from_secs(ENGINE_TIMEOUT_SECS);
    match tokio::time::timeout(timeout, cmd.output()).await {
        Ok(Ok(out)) => Ok(out),
        Ok(Err(e)) => Err(format!("Failed to run command: {e}")),
        Err(_) => Err(format!(
            "Command timed out after {} seconds",
            ENGINE_TIMEOUT_SECS
        )),
    }
}

fn is_direct_executable(program: &str) -> bool {
    let path = Path::new(program);
    if !(path.is_absolute() || program.contains(std::path::MAIN_SEPARATOR)) {
        return false;
    }
    #[cfg(windows)]
    if needs_cmd_wrapper(path) {
        return false;
    }
    true
}

#[cfg(windows)]
fn needs_cmd_wrapper(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("bat") || ext.eq_ignore_ascii_case("cmd"))
}

#[cfg(not(windows))]
fn needs_cmd_wrapper(_path: &Path) -> bool {
    false
}

#[cfg(windows)]
fn wrap_sync(program: &str) -> SyncCommand {
    let mut cmd = SyncCommand::new("cmd");
    cmd.arg("/C").arg(program);
    cmd
}

#[cfg(windows)]
fn wrap_async(program: &str) -> AsyncCommand {
    let mut cmd = AsyncCommand::new("cmd");
    cmd.arg("/C").arg(program);
    cmd
}

#[cfg(not(windows))]
fn wrap_sync(program: &str) -> SyncCommand {
    SyncCommand::new(program)
}

#[cfg(not(windows))]
fn wrap_async(program: &str) -> AsyncCommand {
    AsyncCommand::new(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn bat_files_use_cmd_wrapper() {
        assert!(!is_direct_executable(
            r"C:\project\gradlew.bat"
        ));
    }

    #[test]
    fn exe_files_are_direct() {
        #[cfg(windows)]
        assert!(is_direct_executable(r"C:\tools\trivy.exe"));
        #[cfg(not(windows))]
        assert!(is_direct_executable("/usr/local/bin/trivy"));
    }
}
