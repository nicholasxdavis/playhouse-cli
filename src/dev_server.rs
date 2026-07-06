use std::path::Path;
use std::time::Duration;

use serde::Serialize;
use tokio::process::Command;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize)]
pub struct DevServerInfo {
    pub started: bool,
    pub command: String,
    pub port: u16,
    pub url: String,
    pub stopped: bool,
}

pub struct DevServerHandle {
    child: tokio::process::Child,
}

impl Drop for DevServerHandle {
    fn drop(&mut self) {
        terminate_child(&mut self.child);
    }
}

pub async fn spawn_and_wait(
    cwd: &Path,
    command: &str,
    port: u16,
    timeout_secs: u64,
) -> Result<(DevServerHandle, String), String> {
    let mut child = spawn_shell(cwd, command).await?;
    let url = format!("http://localhost:{port}");
    let attempts = timeout_secs.saturating_mul(2).max(1);
    for _ in 0..attempts {
        if probe_url_async(&url).await {
            return Ok((DevServerHandle { child }, url));
        }
        sleep(Duration::from_millis(500)).await;
    }
    terminate_child(&mut child);
    Err(format!(
        "dev server did not respond on {url} within {timeout_secs}s"
    ))
}

async fn probe_url_async(url: &str) -> bool {
    let url = url.to_string();
    tokio::task::spawn_blocking(move || crate::detect::probe_url(&url))
        .await
        .unwrap_or(false)
}

fn terminate_child(child: &mut tokio::process::Child) {
    if let Some(pid) = child.id() {
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/T", "/F", "/PID", &pid.to_string()])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
        #[cfg(not(windows))]
        {
            let _ = child.start_kill();
        }
    } else {
        let _ = child.start_kill();
    }
    let _ = child.try_wait();
}

async fn spawn_shell(cwd: &Path, command: &str) -> Result<tokio::process::Child, String> {
    #[cfg(windows)]
    let mut cmd = Command::new("cmd");
    #[cfg(windows)]
    cmd.args(["/C", command]);

    #[cfg(not(windows))]
    let mut cmd = Command::new("sh");
    #[cfg(not(windows))]
    cmd.args(["-c", command]);

    cmd.current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    cmd.spawn().map_err(|e| format!("failed to start dev server: {e}"))
}

pub fn resolve_server_port(
    explicit: Option<u16>,
    default_url: Option<&str>,
    workspace: &str,
) -> u16 {
    if let Some(p) = explicit {
        return p;
    }
    if let Some(url) = default_url {
        if let Some(port) = port_from_url(url) {
            return port;
        }
    }
    crate::detect::port_hints(workspace)
        .first()
        .copied()
        .unwrap_or(3000)
}

fn port_from_url(url: &str) -> Option<u16> {
    let rest = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))?;
    let host_port = rest.split('/').next()?;
    host_port.rsplit_once(':').and_then(|(_, p)| p.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_from_url_parses_localhost() {
        assert_eq!(port_from_url("http://localhost:4173/"), Some(4173));
    }

    #[test]
    fn resolve_server_port_prefers_explicit() {
        assert_eq!(
            resolve_server_port(Some(8080), Some("http://localhost:3000"), "."),
            8080
        );
    }
}
