use std::path::Path;
use std::process::Output;

use serde::{Deserialize, Serialize};

use crate::cmd::{r#async as async_cmd, sync};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
    Bun,
}

impl PackageManager {
    pub fn id(self) -> &'static str {
        self.label()
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Yarn => "yarn",
            Self::Bun => "bun",
        }
    }

    pub fn from_id(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "npm" => Some(Self::Npm),
            "pnpm" => Some(Self::Pnpm),
            "yarn" => Some(Self::Yarn),
            "bun" => Some(Self::Bun),
            _ => None,
        }
    }

    pub fn resolve(workspace: &str, setting: &str) -> Self {
        if setting != "auto" {
            if let Some(pm) = Self::from_id(setting) {
                if pm.is_available() {
                    return pm;
                }
            }
        }
        detect_from_lockfiles(workspace).unwrap_or(Self::Npm)
    }

    pub fn is_available(self) -> bool {
        sync(self.program()).arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
    }

    pub fn program(self) -> &'static str {
        match self {
            Self::Npm => crate::cmd::npm_program(),
            Self::Pnpm => {
                #[cfg(windows)]
                {
                    "pnpm.cmd"
                }
                #[cfg(not(windows))]
                {
                    "pnpm"
                }
            }
            Self::Yarn => {
                #[cfg(windows)]
                {
                    "yarn.cmd"
                }
                #[cfg(not(windows))]
                {
                    "yarn"
                }
            }
            Self::Bun => {
                #[cfg(windows)]
                {
                    "bun.exe"
                }
                #[cfg(not(windows))]
                {
                    "bun"
                }
            }
        }
    }

    pub async fn install_resilient(&self, cwd: &Path) -> Result<(), String> {
        const MAX: u32 = 3;
        let mut last = String::new();
        for attempt in 1..=MAX {
            match self.install_inner(cwd).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last = e.clone();
                    if is_fs_lock_error(&e) && node_modules_usable(cwd) {
                        eprintln!(
                            "[!] {} install blocked by file lock; reusing existing node_modules",
                            self.label()
                        );
                        return Ok(());
                    }
                    if attempt < MAX && is_fs_lock_error(&e) {
                        tokio::time::sleep(std::time::Duration::from_millis(400 * attempt as u64))
                            .await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
        Err(last)
    }

    async fn install_inner(&self, cwd: &Path) -> Result<(), String> {
        let out = match self {
            Self::Npm => async_cmd(self.program())
                .args(["install", "--no-fund", "--no-audit"])
                .current_dir(cwd)
                .output()
                .await,
            Self::Pnpm => async_cmd(self.program())
                .args(["install", "--ignore-scripts"])
                .current_dir(cwd)
                .output()
                .await,
            Self::Yarn => async_cmd(self.program())
                .args(["install", "--ignore-scripts"])
                .current_dir(cwd)
                .output()
                .await,
            Self::Bun => async_cmd(self.program())
                .args(["install"])
                .current_dir(cwd)
                .output()
                .await,
        }
        .map_err(|e| e.to_string())?;

        if out.status.success() {
            Ok(())
        } else {
            Err(stderr_or_stdout(&out))
        }
    }

    pub async fn exec(&self, cwd: &Path, bin: &str, args: &[&str]) -> Result<Output, String> {
        self.exec_with_bin_path(cwd, bin, args, None).await
    }

    pub async fn exec_with_bin_path(
        &self,
        cwd: &Path,
        bin: &str,
        args: &[&str],
        npm_dir: Option<&Path>,
    ) -> Result<Output, String> {
        let cmd_args: Vec<String> = match self {
            Self::Npm => {
                let mut v = vec!["exec".into(), "--".into(), bin.into()];
                v.extend(args.iter().map(|s| (*s).to_string()));
                v
            }
            Self::Pnpm => {
                let mut v = vec!["exec".into(), bin.into()];
                v.extend(args.iter().map(|s| (*s).to_string()));
                v
            }
            Self::Yarn => {
                let mut v = vec![bin.into()];
                v.extend(args.iter().map(|s| (*s).to_string()));
                v
            }
            Self::Bun => {
                let mut v = vec!["x".into(), bin.into()];
                v.extend(args.iter().map(|s| (*s).to_string()));
                v
            }
        };

        let refs: Vec<&str> = cmd_args.iter().map(String::as_str).collect();
        let mut cmd = async_cmd(self.program());
        cmd.args(&refs)
            .current_dir(cwd)
            .env("PLAYWRIGHT_HTML_OPEN", "never");
        if let Some(dir) = npm_dir {
            cmd.env("PATH", Self::path_env(dir));
        }
        cmd.output().await.map_err(|e| e.to_string())
    }

    pub async fn install_playwright_browser(&self, cwd: &Path) -> Result<(), String> {
        let out = self
            .exec(cwd, "playwright", &["install", "chromium"])
            .await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(stderr_or_stdout(&out))
        }
    }

    pub async fn run_test_script(&self, cwd: &Path) -> Result<Output, String> {
        self.run_test_script_with(cwd, &[]).await
    }

    pub async fn run_test_script_with(
        &self,
        cwd: &Path,
        extra: &[&str],
    ) -> Result<Output, String> {
        let mut cmd = async_cmd(self.program());
        cmd.arg("test").current_dir(cwd);
        if !extra.is_empty() {
            cmd.arg("--").args(extra);
        }
        cmd.output().await.map_err(|e| e.to_string())
    }

    pub fn path_env(npm_dir: &Path) -> String {
        let bin = npm_dir.join("node_modules").join(".bin");
        #[cfg(windows)]
        let sep = ";";
        #[cfg(not(windows))]
        let sep = ":";
        match std::env::var("PATH") {
            Ok(existing) => format!("{}{}{}", bin.display(), sep, existing),
            Err(_) => bin.to_string_lossy().into_owned(),
        }
    }
}

pub fn detect_from_lockfiles(workspace: &str) -> Option<PackageManager> {
    let scan = crate::workspace::scan_root(workspace);
    detect_from_lockfiles_at(&scan).or_else(|| detect_from_lockfiles_at(Path::new(workspace)))
}

fn detect_from_lockfiles_at(root: &Path) -> Option<PackageManager> {
    if root.join("bun.lockb").is_file() || root.join("bun.lock").is_file() {
        return Some(PackageManager::Bun);
    }
    if root.join("pnpm-lock.yaml").is_file() {
        return Some(PackageManager::Pnpm);
    }
    if root.join("yarn.lock").is_file() {
        return Some(PackageManager::Yarn);
    }
    if root.join("package-lock.json").is_file() || root.join("npm-shrinkwrap.json").is_file() {
        return Some(PackageManager::Npm);
    }
    None
}

fn stderr_or_stdout(out: &Output) -> String {
    let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if !err.is_empty() {
        err.lines().next().unwrap_or(&err).to_string()
    } else {
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .unwrap_or("command failed")
            .to_string()
    }
}

pub fn is_fs_lock_error(err: &str) -> bool {
    let e = err.to_uppercase();
    e.contains("EPERM")
        || e.contains("EBUSY")
        || e.contains("EACCES")
        || e.contains("PERMISSION DENIED")
        || e.contains("ACCESS IS DENIED")
        || e.contains("RESOURCE TEMPORARILY UNAVAILABLE")
        || e.contains("THE PROCESS CANNOT ACCESS THE FILE")
}

fn node_modules_usable(cwd: &Path) -> bool {
    let nm = cwd.join("node_modules");
    if nm.join(".bin").is_dir() {
        return true;
    }
    std::fs::read_dir(&nm)
        .map(|d| d.take(5).count() >= 3)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_windows_lock_errors() {
        assert!(is_fs_lock_error("npm ERR! EPERM: operation not permitted"));
        assert!(is_fs_lock_error("EBUSY: resource temporarily unavailable"));
        assert!(!is_fs_lock_error("package not found"));
    }
}
