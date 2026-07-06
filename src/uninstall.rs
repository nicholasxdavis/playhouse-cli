use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::config::playhouse_home;
use crate::install::{self, InstallProfile};
use crate::tools::{self, playwright_npm_dir};

#[derive(Debug, Clone, Serialize, Default)]
pub struct UninstallReport {
    pub removed: Vec<String>,
    pub failed: Vec<String>,
    pub workspace_npm_removed: bool,
}

pub fn confirm(yes: bool) -> Result<(), String> {
    if yes {
        return Ok(());
    }
    eprint!("Remove Playhouse bundled tools? [y/N] ");
    let mut line = String::new();
    std::io::Write::flush(&mut std::io::stderr()).ok();
    if std::io::stdin().read_line(&mut line).is_err() {
        return Err("Could not read confirmation".into());
    }
    match line.trim().to_lowercase().as_str() {
        "y" | "yes" => Ok(()),
        _ => Err("Uninstall cancelled".into()),
    }
}

pub fn uninstall_global() -> UninstallReport {
    let mut report = UninstallReport::default();
    for path in [tools::bundled_trivy_path(), tools::bundled_arkenar_path()] {
        remove_path(&path, &mut report);
    }
    let cache = playhouse_home().join("cache");
    remove_dir_if_exists(&cache, &mut report);
    report
}

pub fn uninstall_workspace(workspace: &str) -> UninstallReport {
    let mut report = UninstallReport::default();
    let npm_dir = playwright_npm_dir(workspace);
    if npm_dir.is_dir() {
        match fs::remove_dir_all(&npm_dir) {
            Ok(()) => {
                report.removed.push(npm_dir.display().to_string());
                report.workspace_npm_removed = true;
            }
            Err(e) => report
                .failed
                .push(format!("{}: {e}", npm_dir.display())),
        }
    }
    report
}

pub fn uninstall_all(workspace: &str, global: bool, workspace_tools: bool) -> UninstallReport {
    let mut report = UninstallReport::default();
    if global {
        let g = uninstall_global();
        report.removed.extend(g.removed);
        report.failed.extend(g.failed);
    }
    if workspace_tools {
        let w = uninstall_workspace(workspace);
        report.removed.extend(w.removed);
        report.failed.extend(w.failed);
        report.workspace_npm_removed = w.workspace_npm_removed;
    }
    report
}

fn remove_path(path: &Path, report: &mut UninstallReport) {
    if !path.is_file() {
        return;
    }
    match fs::remove_file(path) {
        Ok(()) => report.removed.push(path.display().to_string()),
        Err(e) => report.failed.push(format!("{}: {e}", path.display())),
    }
}

fn remove_dir_if_exists(path: &Path, report: &mut UninstallReport) {
    if !path.is_dir() {
        return;
    }
    match fs::remove_dir_all(path) {
        Ok(()) => report.removed.push(path.display().to_string()),
        Err(e) => report.failed.push(format!("{}: {e}", path.display())),
    }
}

pub fn reinstall_tools_after_update(workspace: &str) -> install::InstallReport {
    // Blocking wrapper for async ensure_profile from sync update path
    tokio::runtime::Handle::try_current()
        .map(|h| {
            h.block_on(install::ensure_profile(
                workspace,
                InstallProfile::Full,
                true,
            ))
        })
        .unwrap_or_else(|_| {
            tokio::runtime::Runtime::new()
                .expect("runtime")
                .block_on(install::ensure_profile(
                    workspace,
                    InstallProfile::Full,
                    true,
                ))
        })
}
