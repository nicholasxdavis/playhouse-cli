use std::path::{Path, PathBuf};

use crate::config::playhouse_home;

pub const TRIVY_VERSION: &str = "0.72.0";
pub const ARKENAR_VERSION: &str = "1.3.2";
pub const PLAYWRIGHT_PKG: &str = "^1.49.0";
pub const LIGHTHOUSE_PKG: &str = "^12.4.0";

pub fn bin_dir() -> PathBuf {
    playhouse_home().join("bin")
}

pub fn playhouse_dir(workspace: &str) -> PathBuf {
    Path::new(workspace).join(".playhouse")
}

pub fn playwright_npm_dir(workspace: &str) -> PathBuf {
    playhouse_dir(workspace).join("npm")
}

#[cfg(windows)]
pub fn trivy_binary_name() -> &'static str {
    "trivy.exe"
}

#[cfg(not(windows))]
pub fn trivy_binary_name() -> &'static str {
    "trivy"
}

pub fn bundled_trivy_path() -> PathBuf {
    bin_dir().join(trivy_binary_name())
}

pub fn has_bundled_trivy() -> bool {
    bundled_trivy_path().is_file()
}

/// Resolve trivy: bundled binary first, then PATH.
pub fn trivy_program() -> String {
    let bundled = bundled_trivy_path();
    if bundled.is_file() {
        bundled.to_string_lossy().into_owned()
    } else {
        "trivy".into()
    }
}

fn node_bin_candidates(root: &Path, name: &str) -> Vec<PathBuf> {
    let bin_dir = root.join("node_modules").join(".bin");
    #[cfg(windows)]
    {
        vec![
            bin_dir.join(format!("{name}.cmd")),
            bin_dir.join(name),
        ]
    }
    #[cfg(not(windows))]
    {
        vec![bin_dir.join(name)]
    }
}

fn first_existing(paths: &[PathBuf]) -> Option<PathBuf> {
    paths.iter().find(|p| p.is_file()).cloned()
}

pub fn project_node_bin(workspace: &str, name: &str) -> Option<PathBuf> {
    let scan = crate::workspace::scan_root(workspace);
    first_existing(&node_bin_candidates(&scan, name))
        .or_else(|| first_existing(&node_bin_candidates(Path::new(workspace), name)))
}

pub fn bundled_node_bin(workspace: &str, name: &str) -> Option<PathBuf> {
    first_existing(&node_bin_candidates(&playwright_npm_dir(workspace), name))
}

/// Where npm/pnpm/yarn/bun exec should run for Playwright/Lighthouse.
#[derive(Debug, Clone)]
pub struct NodeToolContext {
    pub cwd: PathBuf,
    pub npm_prefix: Option<PathBuf>,
    pub source: &'static str,
}

/// Prefer project `node_modules/.bin`, then `.playhouse/npm`.
pub fn resolve_node_tool_context(workspace: &str) -> NodeToolContext {
    let scan = crate::workspace::scan_root(workspace);
    if project_node_bin_at(&scan, "playwright").is_some() {
        return NodeToolContext {
            cwd: scan,
            npm_prefix: None,
            source: "project",
        };
    }
    if project_node_bin_at(Path::new(workspace), "playwright").is_some() {
        return NodeToolContext {
            cwd: Path::new(workspace).to_path_buf(),
            npm_prefix: None,
            source: "project",
        };
    }

    let npm_dir = playwright_npm_dir(workspace);
    NodeToolContext {
        npm_prefix: Some(playwright_prefix(workspace)),
        cwd: npm_dir,
        source: "bundled",
    }
}

fn project_node_bin_at(root: &Path, name: &str) -> Option<PathBuf> {
    first_existing(&node_bin_candidates(root, name))
}

pub fn has_playwright(workspace: &str) -> bool {
    project_node_bin(workspace, "playwright").is_some()
        || bundled_node_bin(workspace, "playwright").is_some()
}

pub fn has_lighthouse(workspace: &str) -> bool {
    project_node_bin(workspace, "lighthouse").is_some()
        || bundled_node_bin(workspace, "lighthouse").is_some()
}

pub fn playwright_prefix(workspace: &str) -> PathBuf {
    playwright_npm_dir(workspace)
}

#[cfg(windows)]
pub fn arkenar_binary_name() -> &'static str {
    "arkenar.exe"
}

#[cfg(not(windows))]
pub fn arkenar_binary_name() -> &'static str {
    "arkenar"
}

pub fn bundled_arkenar_path() -> PathBuf {
    bin_dir().join(arkenar_binary_name())
}

pub fn has_bundled_arkenar() -> bool {
    bundled_arkenar_path().is_file()
}

pub fn arkenar_program() -> String {
    let bundled = bundled_arkenar_path();
    if bundled.is_file() {
        bundled.to_string_lossy().into_owned()
    } else {
        "arkenar".into()
    }
}
