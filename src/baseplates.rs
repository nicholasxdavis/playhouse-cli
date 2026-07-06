use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{load_settings, PlayhouseSettings};
use crate::project::{self, FunctionalRunner, ProjectProfile, ProjectStack};
use crate::tools::playhouse_dir;
use crate::workspace::{self, WorkspaceConfig};

const PLAYWRIGHT_CONFIG_TEMPLATE: &str = include_str!("assets/baseplates/playwright.config.ts");

#[derive(Debug, Clone, Serialize)]
pub struct PlateInfo {
    pub id: String,
    pub label: String,
    pub stack: String,
    pub runner: String,
    pub description: String,
    pub compatible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestDetection {
    pub detected: bool,
    pub paths: Vec<String>,
    pub primary_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScaffoldReport {
    pub plate: String,
    pub files: Vec<String>,
    pub skipped: Vec<String>,
    pub manifest_path: String,
    pub playwright_config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TestManifest {
    version: u8,
    plates: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManifestEntry {
    id: String,
    #[serde(rename = "appliedAt")]
    applied_at: String,
    files: Vec<String>,
}

struct PlateDef {
    id: &'static str,
    label: &'static str,
    stack: &'static str,
    runner: FunctionalRunner,
    description: &'static str,
    template: &'static str,
    filename: &'static str,
    playwright: bool,
}

const PLATES: &[PlateDef] = &[
    PlateDef {
        id: "web-smoke",
        label: "Web smoke",
        stack: "web-e2e",
        runner: FunctionalRunner::Playwright,
        description: "Homepage loads, title present",
        template: include_str!("assets/baseplates/web-smoke.spec.ts"),
        filename: "smoke.spec.ts",
        playwright: true,
    },
    PlateDef {
        id: "web-auth",
        label: "Web auth",
        stack: "web-e2e",
        runner: FunctionalRunner::Playwright,
        description: "Login flow placeholder",
        template: include_str!("assets/baseplates/web-auth.spec.ts"),
        filename: "auth.spec.ts",
        playwright: true,
    },
    PlateDef {
        id: "web-a11y",
        label: "Web a11y",
        stack: "web-e2e",
        runner: FunctionalRunner::Playwright,
        description: "Accessibility smoke checks",
        template: include_str!("assets/baseplates/web-a11y.spec.ts"),
        filename: "a11y.spec.ts",
        playwright: true,
    },
    PlateDef {
        id: "api-health",
        label: "API health",
        stack: "web-e2e",
        runner: FunctionalRunner::Playwright,
        description: "GET /health returns 200",
        template: include_str!("assets/baseplates/api-health.spec.ts"),
        filename: "health.spec.ts",
        playwright: true,
    },
    PlateDef {
        id: "rust-lib",
        label: "Rust lib",
        stack: "rust",
        runner: FunctionalRunner::CargoTest,
        description: "cargo test #[test] fn it_works",
        template: include_str!("assets/baseplates/rust-lib.rs"),
        filename: "playhouse_smoke.rs",
        playwright: false,
    },
    PlateDef {
        id: "python-pytest",
        label: "Python pytest",
        stack: "python",
        runner: FunctionalRunner::Pytest,
        description: "test_import_app starter",
        template: include_str!("assets/baseplates/python-pytest.py"),
        filename: "test_app.py",
        playwright: false,
    },
    PlateDef {
        id: "go-http",
        label: "Go HTTP",
        stack: "go",
        runner: FunctionalRunner::GoTest,
        description: "go test handler stub",
        template: include_str!("assets/baseplates/go-http_test.go"),
        filename: "playhouse_smoke_test.go",
        playwright: false,
    },
];

pub fn list_plates(profile: &ProjectProfile) -> Vec<PlateInfo> {
    PLATES
        .iter()
        .map(|p| PlateInfo {
            id: p.id.into(),
            label: p.label.into(),
            stack: p.stack.into(),
            runner: p.runner.as_str().into(),
            description: p.description.into(),
            compatible: plate_compatible(p, profile),
        })
        .collect()
}

pub fn default_plate_for_profile(profile: &ProjectProfile) -> Option<&'static str> {
    match profile.functional_runner {
        FunctionalRunner::Playwright => Some("web-smoke"),
        FunctionalRunner::CargoTest => Some("rust-lib"),
        FunctionalRunner::Pytest => Some("python-pytest"),
        FunctionalRunner::GoTest => Some("go-http"),
        _ => None,
    }
}

pub fn detect_existing_tests(workspace: &str) -> TestDetection {
    let roots = crate::workspace::resolve_roots(workspace);
    let root = &roots.test;
    let repo = &roots.workspace;
    let mut paths = Vec::new();

    let candidates = [
        root.join("tests").join("e2e"),
        root.join("tests"),
        playhouse_dir(workspace).join("tests"),
        root.join("e2e"),
    ];

    for dir in &candidates {
        if !dir.is_dir() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && is_test_file(&path) {
                    paths.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }

    if root.join("Cargo.toml").is_file() {
        collect_rust_tests(root, &mut paths);
    }
    collect_glob_tests(root, "*.spec.ts", &mut paths);
    collect_glob_tests(root, "*_test.go", &mut paths);
    collect_glob_tests(root, "test_*.py", &mut paths);
    if root != repo {
        collect_glob_tests(repo, "*.spec.ts", &mut paths);
        collect_glob_tests(repo, "*_test.go", &mut paths);
        collect_glob_tests(repo, "test_*.py", &mut paths);
    }

    paths.sort();
    paths.dedup();

    let primary = paths.first().cloned().or_else(|| {
        if root.join("tests").is_dir() {
            Some(root.join("tests").to_string_lossy().into_owned())
        } else if playhouse_dir(workspace).join("tests").is_dir() {
            Some(
                playhouse_dir(workspace)
                    .join("tests")
                    .to_string_lossy()
                    .into_owned(),
            )
        } else {
            None
        }
    });

    TestDetection {
        detected: !paths.is_empty(),
        paths,
        primary_path: primary,
    }
}

pub fn init_plate(
    workspace: &str,
    plate_id: Option<&str>,
    force: bool,
) -> Result<ScaffoldReport, String> {
    let profile = project::detect(workspace);
    let plate_id = plate_id
        .map(String::from)
        .or_else(|| default_plate_for_profile(&profile).map(String::from))
        .ok_or_else(|| "No default baseplate for this stack — pass --plate <id>".to_string())?;

    let detection = detect_existing_tests(workspace);
    if detection.detected && !force {
        return Err(format!(
            "Tests already exist ({}). Use --force to scaffold anyway.",
            detection.paths.join(", ")
        ));
    }

    apply_plate(workspace, &plate_id, force)
}

pub fn add_plate(workspace: &str, plate_id: &str, force: bool) -> Result<ScaffoldReport, String> {
    apply_plate(workspace, plate_id, force)
}

fn apply_plate(workspace: &str, plate_id: &str, force: bool) -> Result<ScaffoldReport, String> {
    let plate = PLATES
        .iter()
        .find(|p| p.id == plate_id)
        .ok_or_else(|| format!("Unknown baseplate '{plate_id}' — run playhouse test list"))?;

    let settings = load_settings();
    let ws = workspace::load_workspace_config(workspace);
    let roots = workspace::resolve_roots(workspace);
    let vars = template_vars(workspace, &settings, &ws, plate);

    let target_dir = target_dir_for_plate(workspace, plate);
    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    let dest = target_dir.join(plate.filename);
    let mut files = Vec::new();
    let mut skipped = Vec::new();

    if dest.is_file() && !force {
        skipped.push(dest.to_string_lossy().into_owned());
    } else {
        let content = render_template(plate.template, &vars);
        fs::write(&dest, content).map_err(|e| e.to_string())?;
        files.push(dest.to_string_lossy().into_owned());
    }

    let mut playwright_config = None;
    if plate.playwright {
        let cfg_path = roots.test.join("playwright.config.ts");
        if !cfg_path.is_file() {
            let test_dir = relative_test_dir(workspace, plate);
            let mut cfg_vars = vars.clone();
            cfg_vars.insert("test_dir".into(), test_dir);
            let content = render_template(PLAYWRIGHT_CONFIG_TEMPLATE, &cfg_vars);
            fs::write(&cfg_path, content).map_err(|e| e.to_string())?;
            playwright_config = Some(cfg_path.to_string_lossy().into_owned());
        }
    }

    let manifest_path = update_manifest(workspace, plate.id, &files)?;

    Ok(ScaffoldReport {
        plate: plate.id.into(),
        files,
        skipped,
        manifest_path,
        playwright_config,
    })
}

fn plate_compatible(plate: &PlateDef, profile: &ProjectProfile) -> bool {
    match plate.runner {
        FunctionalRunner::Playwright => profile.needs_playwright() || profile.browser_audits,
        FunctionalRunner::CargoTest => profile.stack == ProjectStack::Rust,
        FunctionalRunner::Pytest => profile.stack == ProjectStack::Python,
        FunctionalRunner::GoTest => profile.stack == ProjectStack::Go,
        _ => false,
    }
}

fn target_dir_for_plate(workspace: &str, plate: &PlateDef) -> PathBuf {
    let roots = workspace::resolve_roots(workspace);
    let root = &roots.test;
    if plate.playwright {
        if root.join("package.json").is_file() {
            return root.join("tests").join("e2e");
        }
        return playhouse_dir(workspace).join("tests");
    }
    match plate.runner {
        FunctionalRunner::CargoTest => root.join("tests"),
        FunctionalRunner::Pytest => root.join("tests"),
        FunctionalRunner::GoTest => root.to_path_buf(),
        _ => playhouse_dir(workspace).join("tests"),
    }
}

fn relative_test_dir(workspace: &str, plate: &PlateDef) -> String {
    let abs = target_dir_for_plate(workspace, plate);
    let roots = workspace::resolve_roots(workspace);
    abs.strip_prefix(&roots.test)
        .or_else(|_| abs.strip_prefix(&roots.workspace))
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| "./tests/e2e".into())
}

fn template_vars(
    workspace: &str,
    settings: &PlayhouseSettings,
    ws: &WorkspaceConfig,
    _plate: &PlateDef,
) -> std::collections::HashMap<String, String> {
    let url = workspace::resolve_verify_url(workspace, settings)
        .or_else(|| ws.default_url.clone())
        .unwrap_or_else(|| "http://localhost:3000".into());
    let project_name = ws
        .project_name
        .clone()
        .unwrap_or_else(|| workspace::detect_project_name(workspace));

    let mut vars = std::collections::HashMap::new();
    vars.insert("url".into(), url);
    vars.insert("project_name".into(), project_name);
    vars
}

fn render_template(template: &str, vars: &std::collections::HashMap<String, String>) -> String {
    let mut out = template.to_string();
    for (key, value) in vars {
        out = out.replace(&format!("{{{{{key}}}}}"), value);
    }
    out
}

fn manifest_path(workspace: &str) -> PathBuf {
    playhouse_dir(workspace).join("tests").join("manifest.json")
}

fn load_manifest(workspace: &str) -> TestManifest {
    let path = manifest_path(workspace);
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(m) = serde_json::from_str(&content) {
            return m;
        }
    }
    TestManifest {
        version: 1,
        plates: vec![],
    }
}

fn save_manifest(workspace: &str, manifest: &TestManifest) -> Result<(), String> {
    let path = manifest_path(workspace);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(manifest).unwrap_or_default(),
    )
    .map_err(|e| e.to_string())
}

fn update_manifest(workspace: &str, plate_id: &str, files: &[String]) -> Result<String, String> {
    let mut manifest = load_manifest(workspace);
    manifest.plates.retain(|p| p.id != plate_id);
    manifest.plates.push(ManifestEntry {
        id: plate_id.into(),
        applied_at: unix_now(),
        files: files.to_vec(),
    });
    save_manifest(workspace, &manifest)?;
    Ok(manifest_path(workspace).to_string_lossy().into_owned())
}

fn is_test_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name.ends_with(".spec.ts")
        || name.ends_with(".spec.js")
        || name.ends_with("_test.go")
        || name.starts_with("test_") && name.ends_with(".py")
        || name.ends_with(".rs")
}

fn collect_glob_tests(root: &Path, pattern: &str, paths: &mut Vec<String>) {
    fn walk(dir: &Path, pattern: &str, paths: &mut Vec<String>, depth: usize) {
        if depth > 4 {
            return;
        }
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name == "node_modules"
                    || name == "target"
                    || name == ".git"
                    || name == "assets"
                    || name == "baseplates"
                {
                    continue;
                }
                walk(&path, pattern, paths, depth + 1);
            } else if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                if wildcard_match(pattern, fname) {
                    paths.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }
    walk(root, pattern, paths, 0);
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    if let Some(suffix) = pattern.strip_prefix('*') {
        text.ends_with(suffix)
    } else if let Some(prefix) = pattern.strip_suffix('*') {
        text.starts_with(prefix)
    } else {
        pattern == text
    }
}

fn collect_rust_tests(root: &Path, paths: &mut Vec<String>) {
    let tests_dir = root.join("tests");
    if tests_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&tests_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    paths.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }
}

fn unix_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}

pub fn tests_block(workspace: &str, profile: &ProjectProfile) -> serde_json::Value {
    let detection = detect_existing_tests(workspace);
    let manifest = load_manifest(workspace);
    serde_json::json!({
        "detected": detection.detected,
        "paths": detection.paths,
        "testsPath": detection.primary_path,
        "manifestPath": manifest_path(workspace).to_string_lossy(),
        "appliedPlates": manifest.plates,
        "baseplates": list_plates(profile),
        "defaultPlate": default_plate_for_profile(profile),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_substitutes_vars() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("url".into(), "http://localhost:5173".into());
        vars.insert("project_name".into(), "demo".into());
        let out = render_template("goto {{url}} // {{project_name}}", &vars);
        assert!(out.contains("http://localhost:5173"));
        assert!(out.contains("demo"));
    }
}
