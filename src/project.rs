use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// High-level project classification from repo signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectStack {
    WebE2e,
    WebUnit,
    Python,
    Rust,
    Go,
    Java,
    Unknown,
}

impl ProjectStack {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WebE2e => "web-e2e",
            Self::WebUnit => "web-unit",
            Self::Python => "python",
            Self::Rust => "rust",
            Self::Go => "go",
            Self::Java => "java",
            Self::Unknown => "unknown",
        }
    }
}

/// Detected functional test runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FunctionalRunner {
    Playwright,
    NpmTest,
    Pytest,
    CargoTest,
    GoTest,
    MvnTest,
    GradleTest,
    None,
}

impl FunctionalRunner {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Playwright => "playwright",
            Self::NpmTest => "npm-test",
            Self::Pytest => "pytest",
            Self::CargoTest => "cargo-test",
            Self::GoTest => "go-test",
            Self::MvnTest => "mvn-test",
            Self::GradleTest => "gradle-test",
            Self::None => "none",
        }
    }

    pub fn from_config_str(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "playwright" => Some(Self::Playwright),
            "npm-test" | "npm_test" | "npmtest" => Some(Self::NpmTest),
            "pytest" => Some(Self::Pytest),
            "cargo-test" | "cargo_test" | "cargo" => Some(Self::CargoTest),
            "go-test" | "go_test" | "go" => Some(Self::GoTest),
            "mvn-test" | "mvn_test" | "mvn" => Some(Self::MvnTest),
            "gradle-test" | "gradle_test" | "gradle" => Some(Self::GradleTest),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectProfile {
    pub stack: ProjectStack,
    pub functional_runner: FunctionalRunner,
    pub browser_audits: bool,
    pub signals: Vec<String>,
    pub language: String,
}

impl ProjectProfile {
    pub fn needs_node(&self) -> bool {
        self.browser_audits
            || self.needs_playwright()
            || self.functional_runner == FunctionalRunner::NpmTest
    }

    pub fn needs_playwright(&self) -> bool {
        self.functional_runner == FunctionalRunner::Playwright
    }

    pub fn install_profile(&self) -> crate::install::InstallProfile {
        if self.browser_audits || self.needs_playwright() {
            crate::install::InstallProfile::Full
        } else {
            crate::install::InstallProfile::Minimal
        }
    }
}

pub fn detect(workspace: &str) -> ProjectProfile {
    let roots = crate::workspace::resolve_roots(workspace);
    let mut profile = detect_at(&roots.scan);
    apply_workspace_overrides(workspace, &mut profile);
    profile
}

fn apply_workspace_overrides(workspace: &str, profile: &mut ProjectProfile) {
    let cfg = crate::workspace::load_workspace_config(workspace);
    if let Some(ref scan) = cfg.scan_root {
        profile.signals.push(format!("config:scan_root={scan}"));
    }
    if let Some(ref test) = cfg.test_root {
        profile.signals.push(format!("config:test_root={test}"));
    }
    if let Some(ref runner) = cfg.functional_runner {
        if let Some(r) = FunctionalRunner::from_config_str(runner) {
            profile.functional_runner = r;
            profile.signals.push(format!("config:functional_runner={runner}"));
            if r == FunctionalRunner::Playwright {
                profile.stack = ProjectStack::WebE2e;
                profile.browser_audits = true;
                profile.language = "javascript".into();
            }
        }
    }
}

pub fn detect_at(root: &Path) -> ProjectProfile {
    let mut signals = Vec::new();
    detect_from_markers(root, &mut signals)
        .or_else(|| detect_from_package_json(root, &mut signals))
        .unwrap_or_else(|| {
            ProjectProfile {
                stack: ProjectStack::Unknown,
                functional_runner: FunctionalRunner::None,
                browser_audits: false,
                signals,
                language: "unknown".into(),
            }
        })
}

fn make_profile(
    stack: ProjectStack,
    runner: FunctionalRunner,
    browser_audits: bool,
    signals: Vec<String>,
    language: &str,
) -> ProjectProfile {
    ProjectProfile {
        stack,
        functional_runner: runner,
        browser_audits,
        signals,
        language: language.into(),
    }
}

fn detect_from_markers(root: &Path, signals: &mut Vec<String>) -> Option<ProjectProfile> {
    for detect in MARKER_DETECTORS {
        if let Some(profile) = detect(root, signals) {
            return Some(profile);
        }
    }
    None
}

type MarkerDetector = fn(&Path, &mut Vec<String>) -> Option<ProjectProfile>;

const MARKER_DETECTORS: &[MarkerDetector] = &[
    detect_playwright_marker,
    detect_rust_marker,
    detect_go_marker,
    detect_python_marker,
    detect_maven_marker,
    detect_gradle_marker,
];

fn detect_playwright_marker(root: &Path, signals: &mut Vec<String>) -> Option<ProjectProfile> {
    if !has_playwright_config(root) {
        return None;
    }
    signals.push("playwright.config.*".into());
    Some(make_profile(
        ProjectStack::WebE2e,
        FunctionalRunner::Playwright,
        true,
        std::mem::take(signals),
        "javascript",
    ))
}

fn detect_rust_marker(root: &Path, signals: &mut Vec<String>) -> Option<ProjectProfile> {
    if !root.join("Cargo.toml").is_file() {
        return None;
    }
    signals.push("Cargo.toml".into());
    Some(make_profile(
        ProjectStack::Rust,
        FunctionalRunner::CargoTest,
        false,
        std::mem::take(signals),
        "rust",
    ))
}

fn detect_go_marker(root: &Path, signals: &mut Vec<String>) -> Option<ProjectProfile> {
    if !root.join("go.mod").is_file() {
        return None;
    }
    signals.push("go.mod".into());
    Some(make_profile(
        ProjectStack::Go,
        FunctionalRunner::GoTest,
        false,
        std::mem::take(signals),
        "go",
    ))
}

fn detect_python_marker(root: &Path, signals: &mut Vec<String>) -> Option<ProjectProfile> {
    if !has_python_test_layout(root) {
        return None;
    }
    if root.join("pyproject.toml").is_file() {
        signals.push("pyproject.toml".into());
    }
    if root.join("pytest.ini").is_file() {
        signals.push("pytest.ini".into());
    }
    Some(make_profile(
        ProjectStack::Python,
        FunctionalRunner::Pytest,
        false,
        std::mem::take(signals),
        "python",
    ))
}

fn detect_maven_marker(root: &Path, signals: &mut Vec<String>) -> Option<ProjectProfile> {
    if !root.join("pom.xml").is_file() {
        return None;
    }
    signals.push("pom.xml".into());
    Some(make_profile(
        ProjectStack::Java,
        FunctionalRunner::MvnTest,
        false,
        std::mem::take(signals),
        "java",
    ))
}

fn detect_gradle_marker(root: &Path, signals: &mut Vec<String>) -> Option<ProjectProfile> {
    if !has_gradle_layout(root) {
        return None;
    }
    signals.push("gradle".into());
    Some(make_profile(
        ProjectStack::Java,
        FunctionalRunner::GradleTest,
        false,
        std::mem::take(signals),
        "java",
    ))
}

fn detect_from_package_json(root: &Path, signals: &mut Vec<String>) -> Option<ProjectProfile> {
    let pkg = read_package_json(root)?;
    signals.push("package.json".into());
    let browser_app = package_suggests_browser_app(&pkg);

    let (runner, browser_audits) = if has_vitest_or_jest(&pkg) {
        if let Some(fw) = detect_js_test_framework(&pkg) {
            signals.push(fw);
        }
        (FunctionalRunner::NpmTest, browser_app)
    } else if package_has_test_script(&pkg) {
        signals.push("npm test script".into());
        (FunctionalRunner::NpmTest, browser_app)
    } else if browser_app {
        signals.push("web app dependencies".into());
        (FunctionalRunner::None, true)
    } else {
        return None;
    };

    Some(make_profile(
        ProjectStack::WebUnit,
        runner,
        browser_audits,
        std::mem::take(signals),
        "javascript",
    ))
}

pub fn needs_alt_package_manager_checks(workspace: &str, package_manager_setting: &str) -> bool {
    if matches!(package_manager_setting, "pnpm" | "yarn" | "bun") {
        return true;
    }
    let root = Path::new(workspace);
    root.join("pnpm-lock.yaml").is_file()
        || root.join("yarn.lock").is_file()
        || root.join("bun.lockb").is_file()
        || root.join("bun.lock").is_file()
}

fn has_playwright_config(root: &Path) -> bool {
    const NAMES: &[&str] = &[
        "playwright.config.ts",
        "playwright.config.js",
        "playwright.config.mjs",
        "playwright.config.cjs",
    ];
    NAMES.iter().any(|name| root.join(name).is_file())
}

fn has_python_test_layout(root: &Path) -> bool {
    if root.join("pyproject.toml").is_file() || root.join("pytest.ini").is_file() {
        return true;
    }
    if root.join("setup.cfg").is_file() {
        if let Ok(content) = fs::read_to_string(root.join("setup.cfg")) {
            return content.contains("[tool:pytest]") || content.contains("[pytest]");
        }
    }
    root.join("conftest.py").is_file()
        || root.join("tests").is_dir()
            && root.extension().is_none()
            && has_py_files(root.join("tests"))
}

fn has_py_files(dir: PathBuf) -> bool {
    fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|ext| ext == "py")
        })
}

fn has_gradle_layout(root: &Path) -> bool {
    ["build.gradle", "build.gradle.kts", "settings.gradle", "settings.gradle.kts"]
        .iter()
        .any(|name| root.join(name).is_file())
}

fn read_package_json(root: &Path) -> Option<serde_json::Value> {
    let content = fs::read_to_string(root.join("package.json")).ok()?;
    serde_json::from_str(&content).ok()
}

fn dep_keys<'a>(pkg: &'a serde_json::Value, key: &str) -> impl Iterator<Item = &'a str> {
    pkg.get(key)
        .and_then(|v| v.as_object())
        .into_iter()
        .flatten()
        .map(|(k, _)| k.as_str())
}

fn has_vitest_or_jest(pkg: &serde_json::Value) -> bool {
    for key in dep_keys(pkg, "devDependencies").chain(dep_keys(pkg, "dependencies")) {
        if matches!(key, "vitest" | "jest" | "@jest/core" | "@playwright/test") {
            return true;
        }
    }
    false
}

fn detect_js_test_framework(pkg: &serde_json::Value) -> Option<String> {
    for key in dep_keys(pkg, "devDependencies").chain(dep_keys(pkg, "dependencies")) {
        match key {
            "vitest" => return Some("vitest".into()),
            "jest" | "@jest/core" => return Some("jest".into()),
            "@playwright/test" => return Some("@playwright/test".into()),
            _ => {}
        }
    }
    None
}

fn package_has_test_script(pkg: &serde_json::Value) -> bool {
    pkg.get("scripts")
        .and_then(|s| s.get("test"))
        .and_then(|t| t.as_str())
        .is_some_and(|s| !s.is_empty() && s != "echo \"Error: no test specified\" && exit 1")
}

fn package_suggests_browser_app(pkg: &serde_json::Value) -> bool {
    const WEB_DEPS: &[&str] = &[
        "next",
        "vite",
        "react",
        "react-dom",
        "nuxt",
        "astro",
        "@remix-run/react",
        "svelte",
        "@angular/core",
        "vue",
    ];
    for key in dep_keys(pkg, "dependencies")
        .chain(dep_keys(pkg, "devDependencies"))
        .chain(dep_keys(pkg, "peerDependencies"))
    {
        if WEB_DEPS.contains(&key) {
            return true;
        }
    }
    pkg.get("scripts")
        .and_then(|s| s.as_object())
        .is_some_and(|scripts| {
            scripts.keys().any(|k| matches!(k.as_str(), "dev" | "start" | "preview"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_file(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut f = fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn detects_rust_stack() {
        let dir = std::env::temp_dir().join("playhouse-test-rust");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_file(&dir, "Cargo.toml", "[package]\nname = \"demo\"\n");
        let p = detect(dir.to_str().unwrap());
        assert_eq!(p.stack, ProjectStack::Rust);
        assert_eq!(p.functional_runner, FunctionalRunner::CargoTest);
        assert!(!p.browser_audits);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_playwright_stack() {
        let dir = std::env::temp_dir().join("playhouse-test-pw");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_file(&dir, "playwright.config.ts", "export default {};\n");
        let p = detect(dir.to_str().unwrap());
        assert_eq!(p.stack, ProjectStack::WebE2e);
        assert_eq!(p.functional_runner, FunctionalRunner::Playwright);
        assert!(p.browser_audits);
        let _ = fs::remove_dir_all(&dir);
    }
}
