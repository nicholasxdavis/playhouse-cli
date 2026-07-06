use serde::Serialize;
use serde_json::Value;

use crate::cmd::sync;

const DEFAULT_REPO: &str = "nicholasxdavis/playhouse-cli";

#[derive(Debug, Clone, Serialize)]
pub struct UpgradeReport {
    pub current: String,
    pub github_repo: String,
    pub install_method: String,
    pub github: RemoteVersion,
    pub npm: RemoteVersion,
    pub upgrade: UpgradeHints,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoteVersion {
    pub latest: Option<String>,
    pub update_available: bool,
    pub url: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpgradeHints {
    pub npm: String,
    pub cargo: String,
    pub releases: String,
}

pub fn check() -> UpgradeReport {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let repo = std::env::var("PLAYHOUSE_GITHUB_REPO").unwrap_or_else(|_| DEFAULT_REPO.into());
    let install_method = detect_install_method();

    let github = fetch_github_latest(&repo, &current);
    let npm = fetch_npm_latest(&current);

    let releases = format!("https://github.com/{repo}/releases");
    UpgradeReport {
        current: current.clone(),
        github_repo: repo,
        install_method,
        github,
        npm,
        upgrade: UpgradeHints {
            npm: "npm install -g @nicholasxdavis/playhouse-cli@latest".into(),
            cargo: "cargo install playhouse --force".into(),
            releases,
        },
    }
}

fn detect_install_method() -> String {
    if let Ok(method) = std::env::var("PLAYHOUSE_INSTALL_METHOD") {
        return method;
    }
    let exe = std::env::current_exe()
        .ok()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    if exe.contains("node_modules") && exe.contains("playhouse") {
        "npm".into()
    } else if exe.contains("\\target\\") || exe.contains("/target/") {
        "cargo-local".into()
    } else if exe.contains(".cargo") {
        "cargo-install".into()
    } else {
        "path".into()
    }
}

fn fetch_github_latest(repo: &str, current: &str) -> RemoteVersion {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    match http_get(&url) {
        Ok(body) => {
            let tag = serde_json::from_str::<Value>(&body)
                .ok()
                .and_then(|v| v.get("tag_name")?.as_str().map(String::from))
                .map(|t| t.trim_start_matches('v').to_string());
            let latest = tag.clone();
            RemoteVersion {
                update_available: latest
                    .as_ref()
                    .map(|l| version_gt(l, current))
                    .unwrap_or(false),
                latest,
                url: format!("https://github.com/{repo}/releases/latest"),
                error: None,
            }
        }
        Err(e) => RemoteVersion {
            latest: None,
            update_available: false,
            url: format!("https://github.com/{repo}/releases/latest"),
            error: Some(e),
        },
    }
}

fn fetch_npm_latest(current: &str) -> RemoteVersion {
    let url = "https://registry.npmjs.org/@nicholasxdavis%2Fplayhouse-cli/latest";
    match http_get(url) {
        Ok(body) => {
            let latest = serde_json::from_str::<Value>(&body)
                .ok()
                .and_then(|v| v.get("version")?.as_str().map(String::from));
            RemoteVersion {
                update_available: latest
                    .as_ref()
                    .map(|l| version_gt(l, current))
                    .unwrap_or(false),
                latest,
                url: "https://www.npmjs.com/package/@nicholasxdavis/playhouse-cli".into(),
                error: None,
            }
        }
        Err(e) => RemoteVersion {
            latest: None,
            update_available: false,
            url: "https://www.npmjs.com/package/@nicholasxdavis/playhouse-cli".into(),
            error: Some(e),
        },
    }
}

fn http_get(url: &str) -> Result<String, String> {
    #[cfg(windows)]
    {
        let script = format!(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
             (Invoke-WebRequest -Uri '{}' -UseBasicParsing -Headers @{{'User-Agent'='playhouse-cli'}}).Content",
            url.replace('\'', "''")
        );
        let out = sync("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
    #[cfg(not(windows))]
    {
        let out = sync("curl")
            .args([
                "-fsSL",
                "-H",
                "User-Agent: playhouse-cli",
                url,
            ])
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
}

fn version_gt(latest: &str, current: &str) -> bool {
    parse_version(latest) > parse_version(current)
}

fn parse_version(value: &str) -> (u32, u32, u32) {
    let trimmed = value.trim().trim_start_matches('v');
    let mut parts = trimmed.split('.');
    let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let patch = parts
        .next()
        .and_then(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().ok())
        .unwrap_or(0);
    (major, minor, patch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_ordering() {
        assert!(version_gt("0.2.0", "0.1.0"));
        assert!(version_gt("1.0.0", "0.9.9"));
        assert!(!version_gt("0.1.0", "0.1.0"));
        assert!(!version_gt("0.1.0", "0.2.0"));
    }
}
