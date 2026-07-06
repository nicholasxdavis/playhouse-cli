use serde::Serialize;

use crate::cmd::sync;
use crate::uninstall;
use crate::upgrade::{self, UpgradeReport};

#[derive(Debug, Clone, Serialize)]
pub struct UpdateReport {
    pub current: String,
    pub latest: Option<String>,
    pub update_available: bool,
    pub install_method: String,
    pub updated: bool,
    pub message: String,
    pub error: Option<String>,
}

pub fn run_update(workspace: &str) -> UpdateReport {
    let check = upgrade::check();
    let latest = check
        .github
        .latest
        .clone()
        .or_else(|| check.npm.latest.clone());
    let update_available = check.github.update_available || check.npm.update_available;

    if !update_available {
        return UpdateReport {
            current: check.current.clone(),
            latest,
            update_available: false,
            install_method: check.install_method.clone(),
            updated: false,
            message: "Already on the latest version".into(),
            error: None,
        };
    }

    match check.install_method.as_str() {
        "npm" => run_npm_update(&check),
        "cargo-install" => run_cargo_update(&check),
        _ => run_binary_hint(&check, workspace),
    }
}

fn run_npm_update(check: &UpgradeReport) -> UpdateReport {
    let latest = check.github.latest.clone().or_else(|| check.npm.latest.clone());
    let out = sync("npm")
        .args([
            "install",
            "-g",
            "@nicholasxdavis/playhouse-cli@latest",
        ])
        .output();
    match out {
        Ok(o) if o.status.success() => UpdateReport {
            current: check.current.clone(),
            latest,
            update_available: true,
            install_method: check.install_method.clone(),
            updated: true,
            message: "Updated Playhouse via npm".into(),
            error: None,
        },
        Ok(o) => UpdateReport {
            current: check.current.clone(),
            latest,
            update_available: true,
            install_method: check.install_method.clone(),
            updated: false,
            message: "npm update failed".into(),
            error: Some(String::from_utf8_lossy(&o.stderr).trim().to_string()),
        },
        Err(e) => UpdateReport {
            current: check.current.clone(),
            latest,
            update_available: true,
            install_method: check.install_method.clone(),
            updated: false,
            message: "npm update failed".into(),
            error: Some(e.to_string()),
        },
    }
}

fn run_cargo_update(check: &UpgradeReport) -> UpdateReport {
    let latest = check.github.latest.clone();
    let out = sync("cargo")
        .args(["install", "playhouse", "--force", "--locked"])
        .output();
    match out {
        Ok(o) if o.status.success() => UpdateReport {
            current: check.current.clone(),
            latest,
            update_available: true,
            install_method: check.install_method.clone(),
            updated: true,
            message: "Updated Playhouse via cargo install".into(),
            error: None,
        },
        Ok(o) => UpdateReport {
            current: check.current.clone(),
            latest,
            update_available: true,
            install_method: check.install_method.clone(),
            updated: false,
            message: "cargo install failed".into(),
            error: Some(String::from_utf8_lossy(&o.stderr).trim().to_string()),
        },
        Err(e) => UpdateReport {
            current: check.current.clone(),
            latest,
            update_available: true,
            install_method: check.install_method.clone(),
            updated: false,
            message: "cargo install failed".into(),
            error: Some(e.to_string()),
        },
    }
}

fn run_binary_hint(check: &UpgradeReport, workspace: &str) -> UpdateReport {
    let latest = check.github.latest.clone();
    let hint = format!(
        "Download the latest release from {} and replace your playhouse binary, then run `playhouse install`",
        check.upgrade.releases
    );
    let _ = uninstall::reinstall_tools_after_update(workspace);
    UpdateReport {
        current: check.current.clone(),
        latest,
        update_available: true,
        install_method: check.install_method.clone(),
        updated: false,
        message: hint,
        error: None,
    }
}
