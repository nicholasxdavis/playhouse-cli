use std::path::Path;
use std::process::{Command as SyncCommand, Stdio};
use tokio::process::Command as AsyncCommand;

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

fn is_direct_program(program: &str) -> bool {
    matches!(program, "npm" | "npx" | "node" | "npm.cmd" | "npx.cmd")
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

fn is_direct_executable(program: &str) -> bool {
    let path = Path::new(program);
    path.is_absolute() || program.contains(std::path::MAIN_SEPARATOR)
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
