use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayhouseSettings {
    pub last_workspace: Option<String>,
    pub bell_enabled: bool,
    pub desktop_notify_enabled: bool,
    pub auto_run_doctor_on_start: bool,
    pub auto_install_tools: bool,
    pub auto_init_workspace: bool,
    pub default_lighthouse_url: Option<String>,
    pub lighthouse_threshold: f64,
    pub trivy_severity: String,
    pub playwright_reporter: String,
    pub skip_lighthouse_without_server: bool,
    pub skip_lighthouse_in_verify: bool,
    pub skip_playwright_in_verify: bool,
    pub skip_trivy_in_verify: bool,
    pub skip_arkenar_in_verify: bool,
    pub arkenar_advanced_mode: bool,
    pub arkenar_param_fuzz: bool,
    pub arkenar_js_analysis: bool,
    pub arkenar_rate_limit: u32,
    pub arkenar_max_urls: u32,
    pub json_output_default: bool,
    pub light_theme: bool,
    pub accent_color: String,
    pub show_tool_output_in_feed: bool,
    pub auto_export_agent_brief: bool,
    pub max_advisory_history: u32,
    pub stay_on_track_enabled: bool,
    /// Subfolder under .playhouse/ for stay-on-track skill files
    pub stay_on_track_skill_dir: String,
    /// Install .playhouse/SKILL.md for agents (recommended, on by default)
    pub playhouse_skill_enabled: bool,
    /// Subfolder under .playhouse/ for playhouse skill (empty = .playhouse/SKILL.md)
    pub playhouse_skill_dir: String,
    /// Package manager for Playwright/Lighthouse tooling: auto, npm, pnpm, yarn, bun
    pub package_manager: String,
    /// Minimum Playhouse Star Rating (0-100) required to pass verify
    pub star_pass_threshold: u8,
    /// Write .playhouse/AGENT.json after verify or handoff
    pub auto_export_handoff_json: bool,
    /// Agent-friendly defaults (JSON output, workflow hints)
    pub agent_mode: bool,
}

impl Default for PlayhouseSettings {
    fn default() -> Self {
        Self {
            last_workspace: None,
            bell_enabled: true,
            desktop_notify_enabled: false,
            auto_run_doctor_on_start: false,
            auto_install_tools: true,
            auto_init_workspace: true,
            default_lighthouse_url: None,
            lighthouse_threshold: 0.5,
            trivy_severity: "HIGH,CRITICAL".into(),
            playwright_reporter: "json".into(),
            skip_lighthouse_without_server: true,
            skip_lighthouse_in_verify: false,
            skip_playwright_in_verify: false,
            skip_trivy_in_verify: false,
            skip_arkenar_in_verify: false,
            arkenar_advanced_mode: false,
            arkenar_param_fuzz: true,
            arkenar_js_analysis: true,
            arkenar_rate_limit: 50,
            arkenar_max_urls: 40,
            json_output_default: false,
            light_theme: false,
            accent_color: "teal".into(),
            show_tool_output_in_feed: true,
            auto_export_agent_brief: false,
            max_advisory_history: 200,
            stay_on_track_enabled: false,
            stay_on_track_skill_dir: "stay-on-track".into(),
            playhouse_skill_enabled: true,
            playhouse_skill_dir: String::new(),
            package_manager: "auto".into(),
            star_pass_threshold: 75,
            auto_export_handoff_json: true,
            agent_mode: false,
        }
    }
}

pub fn playhouse_home() -> PathBuf {
    ProjectDirs::from("com", "playhouse", "playhouse")
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".playhouse"))
}

pub fn settings_path() -> PathBuf {
    playhouse_home().join("settings.json")
}

pub fn load_settings() -> PlayhouseSettings {
    let path = settings_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(mut settings) = serde_json::from_str::<PlayhouseSettings>(&content) {
            if migrate_skill_paths(&mut settings) {
                save_settings(&settings);
            }
            return settings;
        }
    }
    PlayhouseSettings::default()
}

/// Normalize legacy root-level skill folder names to .playhouse-relative paths.
fn migrate_skill_paths(settings: &mut PlayhouseSettings) -> bool {
    let mut changed = false;
    if settings.playhouse_skill_dir == "PLAYHOUSE" {
        settings.playhouse_skill_dir.clear();
        changed = true;
    }
    if settings.stay_on_track_skill_dir == "STAY-ON-TRACK" {
        settings.stay_on_track_skill_dir = "stay-on-track".into();
        changed = true;
    }
    changed
}

pub fn save_settings(settings: &PlayhouseSettings) {
    let dir = playhouse_home();
    let _ = fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(settings_path(), json);
    }
}

pub fn apply_theme_from_settings(settings: &PlayhouseSettings) {
    crate::tui::set_light_theme(settings.light_theme);
    let _ = crate::tui::set_accent_from_string(&settings.accent_color);
}

/// Config rows per tab: (label, description, enabled)
pub fn config_options_for_tab(tab: usize, settings: &PlayhouseSettings) -> Vec<(String, String, bool)> {
    match tab {
        0 => vec![
            (
                "Bell on task complete".into(),
                "Terminal bell when tasks finish".into(),
                settings.bell_enabled,
            ),
            (
                "Desktop notifications".into(),
                "OS notification on verify complete".into(),
                settings.desktop_notify_enabled,
            ),
            (
                "Light theme".into(),
                "Switch to light terminal theme".into(),
                settings.light_theme,
            ),
            (
                "Show tool output in feed".into(),
                "Detailed engine output in chat feed".into(),
                settings.show_tool_output_in_feed,
            ),
        ],
        1 => vec![
            (
                "Auto-install tools".into(),
                "Install Playwright + Trivy when missing".into(),
                settings.auto_install_tools,
            ),
            (
                "Doctor on startup".into(),
                "Run /doctor when Playhouse opens".into(),
                settings.auto_run_doctor_on_start,
            ),
            (
                "Auto-init workspace".into(),
                "Create .playhouse/ on first run".into(),
                settings.auto_init_workspace,
            ),
            (
                "Skip Lighthouse without server".into(),
                "Don't fail verify when no dev server".into(),
                settings.skip_lighthouse_without_server,
            ),
            (
                "Skip Lighthouse in verify".into(),
                "Run verify without Lighthouse performance audit".into(),
                settings.skip_lighthouse_in_verify,
            ),
            (
                "Skip Playwright in verify".into(),
                "Run verify without functional tests".into(),
                settings.skip_playwright_in_verify,
            ),
            (
                "Skip Trivy in verify".into(),
                "Run verify without security scan".into(),
                settings.skip_trivy_in_verify,
            ),
            (
                "Skip Arkenar in verify".into(),
                "Run verify without DAST web scan".into(),
                settings.skip_arkenar_in_verify,
            ),
            (
                "Star pass threshold 75+".into(),
                "Require 75+ Playhouse Stars to pass verify".into(),
                settings.star_pass_threshold >= 75,
            ),
        ],
        2 => vec![
            (
                "Playhouse agent skill".into(),
                "Install .playhouse/SKILL.md for agents (recommended)".into(),
                settings.playhouse_skill_enabled,
            ),
            (
                "JSON output default".into(),
                "Headless commands default to --json".into(),
                settings.json_output_default,
            ),
            (
                "Agent mode".into(),
                "Optimize defaults for headless agent workflows".into(),
                settings.agent_mode,
            ),
            (
                "Auto-export brief".into(),
                "Write .playhouse/BRIEF.md after verify".into(),
                settings.auto_export_agent_brief,
            ),
            (
                "Auto-export AGENT.json".into(),
                "Write .playhouse/AGENT.json after verify/handoff".into(),
                settings.auto_export_handoff_json,
            ),
        ],
        3 => vec![
            (
                "Arkenar advanced mode".into(),
                "DAST scan profile: advanced vs simple".into(),
                settings.arkenar_advanced_mode,
            ),
            (
                "Arkenar param fuzz".into(),
                "Enable parameter fuzzing during DAST".into(),
                settings.arkenar_param_fuzz,
            ),
            (
                "Arkenar JS analysis".into(),
                "Scan linked scripts for secrets".into(),
                settings.arkenar_js_analysis,
            ),
        ],
        4 => vec![
            (
                "Stay-on-track mode".into(),
                "Spawn .playhouse/stay-on-track/SKILL.md for agents".into(),
                settings.stay_on_track_enabled,
            ),
        ],
        _ => vec![],
    }
}

pub fn config_tab_labels() -> [&'static str; 5] {
    ["General", "Tools", "Agent", "Engines", "Stay-on-track"]
}

pub fn toggle_config_option(settings: &mut PlayhouseSettings, tab: usize, index: usize) {
    match tab {
        0 => match index {
            0 => settings.bell_enabled = !settings.bell_enabled,
            1 => settings.desktop_notify_enabled = !settings.desktop_notify_enabled,
            2 => {
                settings.light_theme = !settings.light_theme;
                apply_theme_from_settings(settings);
            }
            3 => settings.show_tool_output_in_feed = !settings.show_tool_output_in_feed,
            _ => {}
        },
        1 => match index {
            0 => settings.auto_install_tools = !settings.auto_install_tools,
            1 => settings.auto_run_doctor_on_start = !settings.auto_run_doctor_on_start,
            2 => settings.auto_init_workspace = !settings.auto_init_workspace,
            3 => settings.skip_lighthouse_without_server = !settings.skip_lighthouse_without_server,
            4 => settings.skip_lighthouse_in_verify = !settings.skip_lighthouse_in_verify,
            5 => settings.skip_playwright_in_verify = !settings.skip_playwright_in_verify,
            6 => settings.skip_trivy_in_verify = !settings.skip_trivy_in_verify,
            7 => settings.skip_arkenar_in_verify = !settings.skip_arkenar_in_verify,
            8 => {
                settings.star_pass_threshold = if settings.star_pass_threshold >= 75 {
                    60
                } else {
                    75
                };
            }
            _ => {}
        },
        2 => match index {
            0 => settings.playhouse_skill_enabled = !settings.playhouse_skill_enabled,
            1 => settings.json_output_default = !settings.json_output_default,
            2 => settings.agent_mode = !settings.agent_mode,
            3 => settings.auto_export_agent_brief = !settings.auto_export_agent_brief,
            4 => settings.auto_export_handoff_json = !settings.auto_export_handoff_json,
            _ => {}
        },
        3 => match index {
            0 => settings.arkenar_advanced_mode = !settings.arkenar_advanced_mode,
            1 => settings.arkenar_param_fuzz = !settings.arkenar_param_fuzz,
            2 => settings.arkenar_js_analysis = !settings.arkenar_js_analysis,
            _ => {}
        },
        4 if index == 0 => settings.stay_on_track_enabled = !settings.stay_on_track_enabled,
        _ => {}
    }
    save_settings(settings);
}
