use serde_json::{json, Value};

use crate::config::{self, load_settings, save_settings, PlayhouseSettings};
use crate::workspace::{load_workspace_config, save_workspace_config, WorkspaceConfig};

#[derive(Debug, Clone)]
pub struct ConfigKey {
    pub key: &'static str,
    pub scope: &'static str,
    pub kind: &'static str,
    pub description: &'static str,
}

pub fn schema() -> Vec<ConfigKey> {
    vec![
        ConfigKey { key: "package_manager", scope: "global", kind: "string", description: "auto, npm, pnpm, yarn, or bun" },
        ConfigKey { key: "star_pass_threshold", scope: "global", kind: "u8", description: "Min Playhouse Stars (0-100) to pass verify" },
        ConfigKey { key: "lighthouse_threshold", scope: "global", kind: "f64", description: "Min Lighthouse score 0.0-1.0 per category" },
        ConfigKey { key: "default_lighthouse_url", scope: "global", kind: "string|null", description: "Default URL for Lighthouse and Arkenar" },
        ConfigKey { key: "trivy_severity", scope: "global", kind: "string", description: "Trivy severity filter e.g. HIGH,CRITICAL" },
        ConfigKey { key: "json_output_default", scope: "global", kind: "bool", description: "Headless commands default to --json" },
        ConfigKey { key: "auto_install_tools", scope: "global", kind: "bool", description: "Auto-install bundled tools when missing" },
        ConfigKey { key: "auto_export_agent_brief", scope: "global", kind: "bool", description: "Write BRIEF.md after verify" },
        ConfigKey { key: "auto_export_handoff_json", scope: "global", kind: "bool", description: "Write AGENT.json after verify/handoff" },
        ConfigKey { key: "agent_mode", scope: "global", kind: "bool", description: "Prefer JSON output and agent workflow defaults" },
        ConfigKey { key: "skip_playwright_in_verify", scope: "global", kind: "bool", description: "Skip Playwright in verify" },
        ConfigKey { key: "skip_trivy_in_verify", scope: "global", kind: "bool", description: "Skip Trivy in verify" },
        ConfigKey { key: "skip_arkenar_in_verify", scope: "global", kind: "bool", description: "Skip Arkenar DAST in verify" },
        ConfigKey { key: "skip_lighthouse_in_verify", scope: "global", kind: "bool", description: "Skip Lighthouse in verify (Arkenar unaffected)" },
        ConfigKey { key: "skip_lighthouse_without_server", scope: "global", kind: "bool", description: "Allow verify when no URL is available" },
        ConfigKey { key: "stay_on_track_enabled", scope: "global", kind: "bool", description: "Enable stay-on-track skill by default" },
        ConfigKey { key: "playhouse_skill_enabled", scope: "global", kind: "bool", description: "Install .playhouse/SKILL.md for agents (recommended)" },
        ConfigKey { key: "arkenar_advanced_mode", scope: "global", kind: "bool", description: "Arkenar DAST advanced profile" },
        ConfigKey { key: "default_url", scope: "workspace", kind: "string|null", description: "Primary verify URL for Lighthouse, Arkenar, and test baseplates (preferred over global default)" },
        ConfigKey { key: "scan_root", scope: "workspace", kind: "string|null", description: "Monorepo subpath for stack detection and Trivy scans (e.g. apps/web)" },
        ConfigKey { key: "test_root", scope: "workspace", kind: "string|null", description: "Monorepo subpath where functional tests run (defaults to scan_root)" },
        ConfigKey { key: "functional_runner", scope: "workspace", kind: "string|null", description: "Override runner: playwright, cargo-test, go-test, pytest, npm-test, mvn-test, gradle-test, none" },
        ConfigKey { key: "trivy_skip_dirs", scope: "workspace", kind: "string|null", description: "Comma-separated dirs for Trivy --skip-dirs (default: node_modules,.git,vendor)" },
        ConfigKey { key: "audit_headers", scope: "workspace", kind: "object|null", description: "HTTP headers for Lighthouse/Arkenar JSON object e.g. {\"Authorization\":\"Bearer token\"}" },
        ConfigKey { key: "agent_notes", scope: "workspace", kind: "string|null", description: "Notes for agents working in this repo" },
        ConfigKey { key: "project_name", scope: "workspace", kind: "string|null", description: "Display name for this project" },
    ]
}

pub fn schema_json() -> Value {
    json!({
        "precedence": config_precedence(),
        "globalConfigPaths": global_config_paths(),
        "keys": schema().iter().map(|k| {
            let mut entry = json!({
                "key": k.key,
                "scope": k.scope,
                "type": k.kind,
                "description": k.description,
            });
            if let Some(rules) = validation_rules(k.key) {
                entry["validation"] = rules;
            }
            entry
        }).collect::<Vec<_>>(),
    })
}

fn config_precedence() -> Value {
    json!([
        { "setting": "verify URL", "order": ["CLI --url", "workspace.default_url", "global.default_lighthouse_url", "auto-detected local server"] },
        { "setting": "workspace vs global", "order": ["workspace config overrides global for the same key", "CLI flags override both at runtime"] },
        { "setting": "playhouse skill", "order": ["workspace.playhouse_skill OR global.playhouse_skill_enabled"] },
        { "setting": "stay-on-track", "order": ["workspace.stay_on_track OR global.stay_on_track_enabled"] },
    ])
}

fn global_config_paths() -> Value {
    json!({
        "windows": config::settings_path().display().to_string(),
        "macos": "~/.config/playhouse/settings.json",
        "linux": "~/.config/playhouse/settings.json",
        "resolved": config::settings_path(),
        "playhouseHome": config::playhouse_home(),
    })
}

fn validation_rules(key: &str) -> Option<Value> {
    match key {
        "package_manager" => Some(json!({ "enum": ["auto", "npm", "pnpm", "yarn", "bun"] })),
        "star_pass_threshold" => Some(json!({ "min": 0, "max": 100 })),
        "lighthouse_threshold" => Some(json!({ "min": 0.0, "max": 1.0 })),
        "default_lighthouse_url" | "default_url" => {
            Some(json!({ "format": "http-or-https-url" }))
        }
        "functional_runner" => Some(json!({
            "enum": ["playwright", "npm-test", "cargo-test", "go-test", "pytest", "mvn-test", "gradle-test", "none"]
        })),
        "scan_root" | "test_root" => Some(json!({ "mustExist": true, "mustBeDirectory": true })),
        "audit_headers" => Some(json!({
            "format": "json-object",
            "envExpansion": "${VAR} placeholders read from process environment at scan time"
        })),
        _ => None,
    }
}

pub fn get(workspace: &str, key: &str) -> Result<Value, String> {
    let settings = load_settings();
    let ws = load_workspace_config(workspace);
    get_from(&settings, &ws, key)
}

pub fn set(workspace: &str, key: &str, value: &str) -> Result<Value, String> {
    let mut settings = load_settings();
    let mut ws = load_workspace_config(workspace);
    set_on(&mut settings, &mut ws, workspace, key, value)?;
    let out = get_from(&settings, &ws, key)?;
    Ok(out)
}

fn get_from(settings: &PlayhouseSettings, ws: &WorkspaceConfig, key: &str) -> Result<Value, String> {
    match key {
        "package_manager" => Ok(json!(settings.package_manager)),
        "star_pass_threshold" => Ok(json!(settings.star_pass_threshold)),
        "lighthouse_threshold" => Ok(json!(settings.lighthouse_threshold)),
        "default_lighthouse_url" => Ok(json!(settings.default_lighthouse_url)),
        "trivy_severity" => Ok(json!(settings.trivy_severity)),
        "json_output_default" => Ok(json!(settings.json_output_default)),
        "auto_install_tools" => Ok(json!(settings.auto_install_tools)),
        "auto_export_agent_brief" => Ok(json!(settings.auto_export_agent_brief)),
        "auto_export_handoff_json" => Ok(json!(settings.auto_export_handoff_json)),
        "agent_mode" => Ok(json!(settings.agent_mode)),
        "skip_playwright_in_verify" => Ok(json!(settings.skip_playwright_in_verify)),
        "skip_trivy_in_verify" => Ok(json!(settings.skip_trivy_in_verify)),
        "skip_arkenar_in_verify" => Ok(json!(settings.skip_arkenar_in_verify)),
        "skip_lighthouse_in_verify" => Ok(json!(settings.skip_lighthouse_in_verify)),
        "skip_lighthouse_without_server" => Ok(json!(settings.skip_lighthouse_without_server)),
        "stay_on_track_enabled" => Ok(json!(settings.stay_on_track_enabled)),
        "playhouse_skill_enabled" => Ok(json!(settings.playhouse_skill_enabled)),
        "arkenar_advanced_mode" => Ok(json!(settings.arkenar_advanced_mode)),
        "default_url" => Ok(json!(ws.default_url)),
        "scan_root" => Ok(json!(ws.scan_root)),
        "test_root" => Ok(json!(ws.test_root)),
        "functional_runner" => Ok(json!(ws.functional_runner)),
        "trivy_skip_dirs" => Ok(json!(ws.trivy_skip_dirs)),
        "audit_headers" => Ok(json!(ws.audit_headers)),
        "agent_notes" => Ok(json!(ws.agent_notes)),
        "project_name" => Ok(json!(ws.project_name)),
        _ => Err(format!("Unknown key '{key}'. Run `playhouse config schema` for valid keys.")),
    }
}

fn set_on(
    settings: &mut PlayhouseSettings,
    ws: &mut WorkspaceConfig,
    workspace: &str,
    key: &str,
    value: &str,
) -> Result<(), String> {
    let nullish = value.eq_ignore_ascii_case("null") || value.is_empty();

    match key {
        "package_manager" => {
            let v = value.to_lowercase();
            if !matches!(v.as_str(), "auto" | "npm" | "pnpm" | "yarn" | "bun") {
                return Err("package_manager must be auto, npm, pnpm, yarn, or bun".into());
            }
            settings.package_manager = v;
            save_settings(settings);
        }
        "star_pass_threshold" => {
            let v: u8 = value
                .parse()
                .map_err(|_| "star_pass_threshold must be 0-100")?;
            if v > 100 {
                return Err("star_pass_threshold must be 0-100".into());
            }
            settings.star_pass_threshold = v;
            save_settings(settings);
        }
        "lighthouse_threshold" => {
            let v: f64 = value
                .parse()
                .map_err(|_| "lighthouse_threshold must be 0.0-1.0")?;
            if !(0.0..=1.0).contains(&v) {
                return Err("lighthouse_threshold must be 0.0-1.0".into());
            }
            settings.lighthouse_threshold = v;
            save_settings(settings);
        }
        "default_lighthouse_url" => {
            if nullish {
                settings.default_lighthouse_url = None;
            } else {
                crate::workspace::validate_default_url(value)?;
                settings.default_lighthouse_url = Some(value.into());
            }
            save_settings(settings);
        }
        "trivy_severity" => {
            settings.trivy_severity = value.into();
            save_settings(settings);
        }
        "json_output_default" => { settings.json_output_default = parse_bool(value)?; save_settings(settings); }
        "auto_install_tools" => { settings.auto_install_tools = parse_bool(value)?; save_settings(settings); }
        "auto_export_agent_brief" => { settings.auto_export_agent_brief = parse_bool(value)?; save_settings(settings); }
        "auto_export_handoff_json" => { settings.auto_export_handoff_json = parse_bool(value)?; save_settings(settings); }
        "agent_mode" => { settings.agent_mode = parse_bool(value)?; save_settings(settings); }
        "skip_playwright_in_verify" => { settings.skip_playwright_in_verify = parse_bool(value)?; save_settings(settings); }
        "skip_trivy_in_verify" => { settings.skip_trivy_in_verify = parse_bool(value)?; save_settings(settings); }
        "skip_arkenar_in_verify" => { settings.skip_arkenar_in_verify = parse_bool(value)?; save_settings(settings); }
        "skip_lighthouse_in_verify" => { settings.skip_lighthouse_in_verify = parse_bool(value)?; save_settings(settings); }
        "skip_lighthouse_without_server" => { settings.skip_lighthouse_without_server = parse_bool(value)?; save_settings(settings); }
        "stay_on_track_enabled" => { settings.stay_on_track_enabled = parse_bool(value)?; save_settings(settings); }
        "playhouse_skill_enabled" => { settings.playhouse_skill_enabled = parse_bool(value)?; save_settings(settings); }
        "arkenar_advanced_mode" => { settings.arkenar_advanced_mode = parse_bool(value)?; save_settings(settings); }
        "default_url" => {
            if nullish {
                ws.default_url = None;
            } else {
                crate::workspace::validate_default_url(value)?;
                ws.default_url = Some(value.into());
            }
            save_workspace_config(workspace, ws).map_err(|e| e.to_string())?;
        }
        "scan_root" => {
            ws.scan_root = if nullish { None } else { Some(value.into()) };
            validate_workspace_subpath(workspace, ws.scan_root.as_deref())?;
            save_workspace_config(workspace, ws).map_err(|e| e.to_string())?;
        }
        "test_root" => {
            ws.test_root = if nullish { None } else { Some(value.into()) };
            validate_workspace_subpath(workspace, ws.test_root.as_deref())?;
            save_workspace_config(workspace, ws).map_err(|e| e.to_string())?;
        }
        "functional_runner" => {
            if nullish {
                ws.functional_runner = None;
            } else if crate::project::FunctionalRunner::from_config_str(value).is_some() {
                ws.functional_runner = Some(value.into());
            } else {
                return Err("functional_runner must be playwright, npm-test, cargo-test, go-test, pytest, mvn-test, gradle-test, or none".into());
            }
            save_workspace_config(workspace, ws).map_err(|e| e.to_string())?;
        }
        "trivy_skip_dirs" => {
            ws.trivy_skip_dirs = if nullish { None } else { Some(value.into()) };
            save_workspace_config(workspace, ws).map_err(|e| e.to_string())?;
        }
        "audit_headers" => {
            if nullish {
                ws.audit_headers = None;
            } else {
                ws.audit_headers = Some(crate::workspace::parse_audit_headers(value)?);
            }
            save_workspace_config(workspace, ws).map_err(|e| e.to_string())?;
        }
        "agent_notes" => {
            ws.agent_notes = if nullish { None } else { Some(value.into()) };
            save_workspace_config(workspace, ws).map_err(|e| e.to_string())?;
        }
        "project_name" => {
            ws.project_name = if nullish { None } else { Some(value.into()) };
            save_workspace_config(workspace, ws).map_err(|e| e.to_string())?;
        }
        _ => return Err(format!("Unknown key '{key}'")),
    }
    Ok(())
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(format!("Expected true/false, got '{value}'")),
    }
}

fn validate_workspace_subpath(workspace: &str, relative: Option<&str>) -> Result<(), String> {
    crate::workspace::validate_workspace_subpath(workspace, relative)
}
