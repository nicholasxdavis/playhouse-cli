use std::fs;
use std::path::{Path, PathBuf};

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::PlayhouseSettings;
use crate::install;
use crate::tools::playhouse_dir;

const SKILL_TEMPLATE: &str = include_str!("assets/stay_on_track_skill.md");
const PLAYHOUSE_SKILL_TEMPLATE: &str = include_str!("assets/playhouse_skill.md");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspaceConfig {
    pub stay_on_track: bool,
    pub playhouse_skill: bool,
    pub initialized: bool,
    pub project_name: Option<String>,
    /// Primary verify URL for Lighthouse, Arkenar, and test baseplates.
    pub default_url: Option<String>,
    /// Relative path from repo root for stack detection and Trivy scans (monorepo apps).
    pub scan_root: Option<String>,
    /// Relative path from repo root where functional tests run (defaults to scan_root).
    pub test_root: Option<String>,
    /// Override auto-detected functional runner (e.g. playwright, cargo-test).
    pub functional_runner: Option<String>,
    /// Comma-separated dirs for Trivy --skip-dirs (default: node_modules,.git,vendor).
    pub trivy_skip_dirs: Option<String>,
    /// HTTP headers for Lighthouse and Arkenar (e.g. Authorization). Do not commit secrets.
    pub audit_headers: Option<HashMap<String, String>>,
    pub agent_notes: Option<String>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            stay_on_track: false,
            playhouse_skill: true,
            initialized: false,
            project_name: None,
            default_url: None,
            scan_root: None,
            test_root: None,
            functional_runner: None,
            trivy_skip_dirs: None,
            audit_headers: None,
            agent_notes: None,
        }
    }
}

const DEFAULT_TRIVY_SKIP_DIRS: &str = "node_modules,.git,vendor";

/// Directories passed to Trivy `--skip-dirs` (dot-directories like `.well-known` are not skipped).
pub fn trivy_skip_dirs(workspace: &str) -> Vec<String> {
    let ws = load_workspace_config(workspace);
    let raw = ws
        .trivy_skip_dirs
        .as_deref()
        .unwrap_or(DEFAULT_TRIVY_SKIP_DIRS);
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn audit_headers(workspace: &str) -> Option<HashMap<String, String>> {
    load_workspace_config(workspace).audit_headers
}

/// JSON string for Lighthouse `--extra-headers`.
pub fn lighthouse_extra_headers_json(headers: &HashMap<String, String>) -> String {
    serde_json::to_string(headers).unwrap_or_else(|_| "{}".into())
}

/// Arkenar `--header "Name: value"` argument pairs.
pub fn arkenar_header_args(headers: &HashMap<String, String>) -> Vec<String> {
    let mut args = Vec::new();
    for (name, value) in headers {
        args.push("--header".into());
        args.push(format!("{name}: {value}"));
    }
    args
}

/// Validate audit_headers JSON object on config set.
pub fn parse_audit_headers(value: &str) -> Result<HashMap<String, String>, String> {
    let v: serde_json::Value = serde_json::from_str(value)
        .map_err(|_| "audit_headers must be JSON, e.g. {\"Authorization\":\"Bearer token\"}".to_string())?;
    let Some(obj) = v.as_object() else {
        return Err("audit_headers must be a JSON object".into());
    };
    if obj.is_empty() {
        return Err("audit_headers cannot be empty".into());
    }
    let mut map = HashMap::new();
    for (k, val) in obj {
        if k.trim().is_empty() {
            return Err("audit_headers keys cannot be empty".into());
        }
        let s = val
            .as_str()
            .ok_or_else(|| format!("audit_headers value for '{k}' must be a string"))?;
        if s.is_empty() {
            return Err(format!("audit_headers value for '{k}' cannot be empty"));
        }
        map.insert(k.clone(), s.to_string());
    }
    Ok(map)
}

/// Resolved filesystem roots for a workspace (repo root vs monorepo sub-packages).
#[derive(Debug, Clone)]
pub struct WorkspaceRoots {
    pub workspace: PathBuf,
    pub scan: PathBuf,
    pub test: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct InitReport {
    pub workspace: String,
    pub playhouse_dir: String,
    pub tools: install::InstallReport,
    pub stay_on_track: bool,
    pub playhouse_skill: bool,
    pub skill_path: Option<String>,
    pub playhouse_skill_path: Option<String>,
    pub brief_path: String,
}

pub fn workspace_config_path(workspace: &str) -> PathBuf {
    playhouse_dir(workspace).join("config.json")
}

pub fn load_workspace_config(workspace: &str) -> WorkspaceConfig {
    let path = workspace_config_path(workspace);
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str(&content) {
            return cfg;
        }
    }
    WorkspaceConfig::default()
}

pub fn save_workspace_config(workspace: &str, config: &WorkspaceConfig) -> std::io::Result<()> {
    let dir = playhouse_dir(workspace);
    fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(config).unwrap_or_default();
    fs::write(workspace_config_path(workspace), json)
}

pub fn skill_dir(workspace: &str, settings: &PlayhouseSettings) -> PathBuf {
    skill_subdir(playhouse_dir(workspace), &settings.stay_on_track_skill_dir)
}

pub fn skill_path(workspace: &str, settings: &PlayhouseSettings) -> PathBuf {
    skill_dir(workspace, settings).join("SKILL.md")
}

pub fn project_info_path(workspace: &str, settings: &PlayhouseSettings) -> PathBuf {
    skill_dir(workspace, settings).join("PROJECT.md")
}

pub fn playhouse_skill_dir(workspace: &str, settings: &PlayhouseSettings) -> PathBuf {
    skill_subdir(playhouse_dir(workspace), &settings.playhouse_skill_dir)
}

pub fn playhouse_skill_path(workspace: &str, settings: &PlayhouseSettings) -> PathBuf {
    playhouse_skill_dir(workspace, settings).join("SKILL.md")
}

fn skill_subdir(base: PathBuf, sub: &str) -> PathBuf {
    if sub.is_empty() || sub == "." {
        base
    } else {
        base.join(sub)
    }
}

pub fn resolve_roots(workspace: &str) -> WorkspaceRoots {
    let workspace_path = Path::new(workspace);
    let cfg = load_workspace_config(workspace);
    let scan = resolve_subpath(workspace_path, cfg.scan_root.as_deref())
        .unwrap_or_else(|_| workspace_path.to_path_buf());
    let test = resolve_subpath(
        workspace_path,
        cfg.test_root
            .as_deref()
            .or(cfg.scan_root.as_deref()),
    )
    .unwrap_or_else(|_| scan.clone());
    WorkspaceRoots {
        workspace: workspace_path.to_path_buf(),
        scan,
        test,
    }
}

pub fn scan_root(workspace: &str) -> PathBuf {
    resolve_roots(workspace).scan
}

pub fn test_root(workspace: &str) -> PathBuf {
    resolve_roots(workspace).test
}

pub fn scan_root_str(workspace: &str) -> String {
    scan_root(workspace).to_string_lossy().into_owned()
}

pub fn test_root_str(workspace: &str) -> String {
    test_root(workspace).to_string_lossy().into_owned()
}

pub fn validate_workspace_subpath(workspace: &str, relative: Option<&str>) -> Result<(), String> {
    let path = resolve_subpath(Path::new(workspace), relative)?;
    if let Some(rel) = relative {
        let rel = rel.trim();
        if !rel.is_empty() {
            let meta = std::fs::metadata(&path)
                .map_err(|_| format!("path does not exist: {rel}"))?;
            if !meta.is_dir() {
                return Err(format!("path is not a directory: {rel}"));
            }
        }
    }
    Ok(())
}

/// Validate a workspace verify URL before saving to config.
pub fn validate_default_url(url: &str) -> Result<(), String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("default_url cannot be empty".into());
    }
    let rest = if let Some(r) = url.strip_prefix("https://") {
        r
    } else if let Some(r) = url.strip_prefix("http://") {
        r
    } else {
        return Err("default_url must start with http:// or https://".into());
    };
    let host_port = rest.split('/').next().unwrap_or(rest);
    if host_port.is_empty() || host_port.contains(' ') {
        return Err("default_url is not a valid URL".into());
    }
    Ok(())
}

fn resolve_subpath(workspace: &Path, relative: Option<&str>) -> Result<PathBuf, String> {
    let Some(rel) = relative else {
        return Ok(workspace.to_path_buf());
    };
    let rel = rel.trim().trim_start_matches("./").replace('\\', "/");
    if rel.is_empty() {
        return Ok(workspace.to_path_buf());
    }
    if rel.contains("..") {
        return Err("path cannot contain ..".into());
    }
    let joined = workspace.join(&rel);
    Ok(joined)
}

pub fn playhouse_skill_enabled(workspace: &str, settings: &PlayhouseSettings) -> bool {
    let ws = load_workspace_config(workspace);
    ws.playhouse_skill || settings.playhouse_skill_enabled
}

pub fn resolve_verify_url(workspace: &str, settings: &PlayhouseSettings) -> Option<String> {
    let ws = load_workspace_config(workspace);
    ws.default_url
        .or_else(|| settings.default_lighthouse_url.clone())
        .or_else(|| crate::detect::find_local_server(&scan_root_str(workspace)))
}

pub fn agent_json_path(workspace: &str) -> PathBuf {
    playhouse_dir(workspace).join("AGENT.json")
}

pub fn detect_project_name(workspace: &str) -> String {
    let root = Path::new(workspace);
    if let Ok(content) = fs::read_to_string(root.join("package.json")) {
        if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = pkg.get("name").and_then(|n| n.as_str()) {
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }
    root.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string()
}

pub async fn init_workspace(
    workspace: &str,
    settings: &PlayhouseSettings,
    install_tools: bool,
    enable_stay_on_track: bool,
    quiet: bool,
) -> Result<InitReport, String> {
    let dir = playhouse_dir(workspace);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(dir.join("reports")).map_err(|e| e.to_string())?;

    let tools = if install_tools {
        install::ensure_all(workspace, quiet).await
    } else {
        install::InstallReport::default()
    };

    let project_name = detect_project_name(workspace);
    let mut ws_config = load_workspace_config(workspace);
    ws_config.initialized = true;
    ws_config.project_name = Some(project_name.clone());
    if enable_stay_on_track {
        ws_config.stay_on_track = true;
    }
    if settings.playhouse_skill_enabled {
        ws_config.playhouse_skill = true;
    }
    save_workspace_config(workspace, &ws_config).map_err(|e| e.to_string())?;

    let skill = if enable_stay_on_track || settings.stay_on_track_enabled {
        Some(enable_stay_on_track_mode(workspace, settings)?)
    } else {
        None
    };

    let playhouse_skill = if ws_config.playhouse_skill || settings.playhouse_skill_enabled {
        Some(install_playhouse_skill(workspace, settings)?)
    } else {
        None
    };

    let brief_path = dir.join("BRIEF.md");
    let brief = crate::agent::build_brief_text(workspace, settings, &ws_config);
    fs::write(&brief_path, brief).map_err(|e| e.to_string())?;

    Ok(InitReport {
        workspace: workspace.to_string(),
        playhouse_dir: dir.to_string_lossy().into_owned(),
        tools,
        stay_on_track: ws_config.stay_on_track,
        playhouse_skill: ws_config.playhouse_skill || settings.playhouse_skill_enabled,
        skill_path: skill.map(|p| p.to_string_lossy().into_owned()),
        playhouse_skill_path: playhouse_skill.map(|p| p.to_string_lossy().into_owned()),
        brief_path: brief_path.to_string_lossy().into_owned(),
    })
}

pub fn enable_stay_on_track_mode(
    workspace: &str,
    settings: &PlayhouseSettings,
) -> Result<PathBuf, String> {
    let dir = skill_dir(workspace, settings);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let project_name = detect_project_name(workspace);
    let skill_dest = skill_path(workspace, settings);
    let project_dest = project_info_path(workspace, settings);

    let skill_body = build_skill_document(workspace, settings, &project_name);
    fs::write(&skill_dest, skill_body).map_err(|e| e.to_string())?;

    let project_body = build_project_template(&project_name);
    if !project_dest.exists() {
        fs::write(&project_dest, project_body).map_err(|e| e.to_string())?;
    }

    let mut ws_config = load_workspace_config(workspace);
    ws_config.stay_on_track = true;
    ws_config.project_name = Some(project_name);
    save_workspace_config(workspace, &ws_config).map_err(|e| e.to_string())?;

    Ok(skill_dest)
}

pub fn disable_stay_on_track_mode(workspace: &str) -> Result<(), String> {
    let mut ws_config = load_workspace_config(workspace);
    ws_config.stay_on_track = false;
    save_workspace_config(workspace, &ws_config).map_err(|e| e.to_string())
}

/// Write stay-on-track skill files when the workspace flag is set but files are missing.
pub fn repair_stay_on_track(
    workspace: &str,
    settings: &PlayhouseSettings,
    ws_config: &mut WorkspaceConfig,
) {
    if !(ws_config.stay_on_track || settings.stay_on_track_enabled) {
        return;
    }
    if skill_path(workspace, settings).is_file() {
        return;
    }
    if let Ok(path) = enable_stay_on_track_mode(workspace, settings) {
        ws_config.stay_on_track = true;
        let _ = path;
    }
}

pub fn stay_on_track_status(workspace: &str, settings: &PlayhouseSettings) -> serde_json::Value {
    let ws = load_workspace_config(workspace);
    let skill = skill_path(workspace, settings);
    serde_json::json!({
        "enabled": ws.stay_on_track || settings.stay_on_track_enabled,
        "workspaceFlag": ws.stay_on_track,
        "globalSetting": settings.stay_on_track_enabled,
        "skillPath": skill.to_string_lossy(),
        "skillExists": skill.is_file(),
        "skillDir": format!(".playhouse/{}", settings.stay_on_track_skill_dir),
        "projectInfoPath": project_info_path(workspace, settings).to_string_lossy(),
    })
}

pub fn install_playhouse_skill(
    workspace: &str,
    settings: &PlayhouseSettings,
) -> Result<PathBuf, String> {
    let dir = playhouse_skill_dir(workspace, settings);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let project_name = detect_project_name(workspace);
    let skill_dest = playhouse_skill_path(workspace, settings);
    let body = build_playhouse_skill_document(workspace, settings, &project_name);
    fs::write(&skill_dest, body).map_err(|e| e.to_string())?;

    let mut ws_config = load_workspace_config(workspace);
    ws_config.playhouse_skill = true;
    if ws_config.project_name.is_none() {
        ws_config.project_name = Some(project_name);
    }
    save_workspace_config(workspace, &ws_config).map_err(|e| e.to_string())?;

    Ok(skill_dest)
}

pub fn disable_playhouse_skill_mode(workspace: &str) -> Result<(), String> {
    let mut ws_config = load_workspace_config(workspace);
    ws_config.playhouse_skill = false;
    save_workspace_config(workspace, &ws_config).map_err(|e| e.to_string())
}

pub fn playhouse_skill_status(workspace: &str, settings: &PlayhouseSettings) -> serde_json::Value {
    let ws = load_workspace_config(workspace);
    let skill = playhouse_skill_path(workspace, settings);
    serde_json::json!({
        "enabled": ws.playhouse_skill || settings.playhouse_skill_enabled,
        "recommended": true,
        "workspaceFlag": ws.playhouse_skill,
        "globalSetting": settings.playhouse_skill_enabled,
        "skillPath": skill.to_string_lossy(),
        "skillExists": skill.is_file(),
        "skillDir": if settings.playhouse_skill_dir.is_empty() {
            ".playhouse".to_string()
        } else {
            format!(".playhouse/{}", settings.playhouse_skill_dir)
        },
    })
}

fn build_skill_document(workspace: &str, settings: &PlayhouseSettings, project_name: &str) -> String {
    let project_file = project_info_path(workspace, settings);
    let skill_dir = format!(".playhouse/{}", settings.stay_on_track_skill_dir);
    let footer = format!(
        r#"

---

## Playhouse stay-on-track (auto-generated)

**Workspace:** `{workspace}`
**Project:** {project_name}
**Skill folder:** `{dir}`

### Agent instructions (read this first)

1. Load and follow this skill before making any code changes.
2. Open `{project_file}` and complete every section with the user before coding.
3. Output the **Project Info**, **Project Rules** (8 rules), and **Deployment Verification** sections from the workflow above.
4. Use Playhouse headless CLI for all QA (same commands a human would run):

```bash
playhouse doctor --json
playhouse verify --json
playhouse playwright --json
playhouse trivy --json
playhouse lighthouse --json
playhouse agent --json
playhouse stay-on-track status --json
```

5. Review exit codes and JSON output. Fix failures before marking work complete.
6. Never skip `playhouse verify` before deployment or handoff.

### Paths agents should read

| File | Purpose |
|------|---------|
| `{project_file}` | Fill project info + rules with user |
| `.playhouse/BRIEF.md` | Workspace QA brief |
| `.playhouse/advisories.log` | Human notes from TUI |
| `playhouse agent --json` | Machine-readable full reference |
"#,
        dir = skill_dir,
        project_file = project_file.display(),
    );
    format!("{SKILL_TEMPLATE}{footer}")
}

fn build_playhouse_skill_document(
    workspace: &str,
    settings: &PlayhouseSettings,
    project_name: &str,
) -> String {
    let footer = format!(
        r#"

---

## Playhouse workspace (auto-generated)

**Workspace:** `{workspace}`
**Project:** {project_name}
**Skill folder:** `{dir}`

Agents: read this skill first, then run `playhouse agent --json`.
"#,
        dir = if settings.playhouse_skill_dir.is_empty() {
            ".playhouse".into()
        } else {
            format!(".playhouse/{}", settings.playhouse_skill_dir)
        },
    );
    format!("{PLAYHOUSE_SKILL_TEMPLATE}{footer}")
}

fn build_project_template(project_name: &str) -> String {
    format!(
        r#"# Project Info (complete with user before coding)

name: {project_name}
scope: [ASK USER - what is in scope for this session?]
end-goal: [ASK USER - what does done look like?]
project-notes: [AGENT - summarize UI trends, stack, and guardrails from codebase review]

---

# Project Rules (8 rules - agent drafts from codebase, user confirms)

1. [Rule 1]
2. [Rule 2]
3. [Rule 3]
4. [Rule 4]
5. [Rule 5]
6. [Rule 6]
7. [Rule 7]
8. [Rule 8]

---

# Deployment Verification

Before any deployment or handoff:

- [ ] Run `playhouse verify --json` and review all results
- [ ] Fix Playwright failures
- [ ] Fix Trivy HIGH/CRITICAL findings and secrets
- [ ] Lighthouse scores meet threshold (see `.playhouse/BRIEF.md`)
- [ ] User confirmed project rules are still accurate
"#
    )
}

pub fn maybe_auto_init(workspace: &str, settings: &PlayhouseSettings) {
    if !settings.auto_init_workspace {
        return;
    }
    let dir = playhouse_dir(workspace);
    if dir.is_dir() {
        if settings.playhouse_skill_enabled && playhouse_skill_enabled(workspace, settings) {
            let skill = playhouse_skill_path(workspace, settings);
            if !skill.is_file() {
                let _ = install_playhouse_skill(workspace, settings);
            }
        }
        return;
    }
    let _ = fs::create_dir_all(dir.join("reports"));
    let _ = fs::create_dir_all(dir.join("tests"));
    let cfg = WorkspaceConfig {
        initialized: true,
        project_name: Some(detect_project_name(workspace)),
        ..Default::default()
    };
    let _ = save_workspace_config(workspace, &cfg);
    if settings.playhouse_skill_enabled {
        let _ = install_playhouse_skill(workspace, settings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_roots_defaults_to_workspace() {
        let dir = std::env::temp_dir().join(format!("playhouse-roots-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let roots = resolve_roots(dir.to_str().unwrap());
        assert_eq!(roots.workspace, dir);
        assert_eq!(roots.scan, dir);
        assert_eq!(roots.test, dir);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_roots_honors_scan_and_test() {
        let dir = std::env::temp_dir().join(format!("playhouse-mono-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let app = dir.join("apps").join("web");
        std::fs::create_dir_all(&app).unwrap();
        let cfg = WorkspaceConfig {
            scan_root: Some("apps/web".into()),
            test_root: Some("apps/web".into()),
            ..Default::default()
        };
        save_workspace_config(dir.to_str().unwrap(), &cfg).unwrap();
        let roots = resolve_roots(dir.to_str().unwrap());
        assert_eq!(roots.scan, app);
        assert_eq!(roots.test, app);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_parent_escape() {
        let dir = std::env::temp_dir().join(format!("playhouse-escape-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(validate_workspace_subpath(dir.to_str().unwrap(), Some("../outside")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_default_url_requires_http_scheme() {
        assert!(validate_default_url("http://localhost:3000").is_ok());
        assert!(validate_default_url("abc").is_err());
    }

    #[test]
    fn validate_subpath_requires_existing_dir() {
        let dir = std::env::temp_dir().join(format!("playhouse-val-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(validate_workspace_subpath(dir.to_str().unwrap(), Some("missing")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_audit_headers_accepts_json_object() {
        let map = parse_audit_headers(r#"{"Authorization":"Bearer tok"}"#).unwrap();
        assert_eq!(map.get("Authorization").map(String::as_str), Some("Bearer tok"));
        assert!(parse_audit_headers("[]").is_err());
        assert!(parse_audit_headers("{}").is_err());
    }
}
