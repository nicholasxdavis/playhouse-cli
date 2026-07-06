use tokio::sync::mpsc;

use crate::agent;
use crate::auth;
use crate::baseplates;
use crate::config_cli;
use crate::detect;
use crate::project;
use crate::score;
use crate::tui::app::{App, AppMode, FeedRole, TaskKind, VerifyParams};
use crate::tui::config;
use crate::tui::tasks::{spawn_task, TaskEvent};
use crate::tui::ui_blocks::ContentBlock;
use crate::uninstall;
use crate::update;
use crate::upgrade;
use crate::verify_progress;
use crate::workspace;

pub fn submit_input(app: &mut App, task_tx: &mpsc::UnboundedSender<TaskEvent>) {
    let text = app.input_text.trim().to_string();
    app.input_text.clear();
    app.cursor_position = 0;
    app.input_select_anchor = None;
    if text.is_empty() {
        return;
    }
    if text.starts_with('/') {
        execute_command(app, &text, task_tx);
    } else if let Some(cmd) = natural_to_slash(&text) {
        execute_command(app, &cmd, task_tx);
    } else if app.is_busy() {
        app.push_system("Wait for the current task to finish.");
    } else {
        app.push_advisory(&text);
    }
}

pub fn execute_command(app: &mut App, command: &str, task_tx: &mpsc::UnboundedSender<TaskEvent>) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let cmd = parts.first().copied().unwrap_or(command).to_lowercase();
    match cmd.as_str() {
        "/doctor" => {
            let resolve = parts
                .get(1)
                .is_some_and(|s| s.eq_ignore_ascii_case("resolve"));
            start_task(app, task_tx, TaskKind::Doctor { resolve });
        }
        "/install" => start_task(app, task_tx, TaskKind::Install),
        "/init" => start_task(
            app,
            task_tx,
            TaskKind::Init {
                stay_on_track: parts.contains(&"--stay-on-track")
                    || app.settings.stay_on_track_enabled,
                no_skill: parts.contains(&"--no-skill"),
            },
        ),
        "/verify" => start_task(
            app,
            task_tx,
            TaskKind::Verify {
                params: parse_verify_flags(&parts),
            },
        ),
        "/score" => {
            if parts
                .get(1)
                .is_some_and(|s| s.eq_ignore_ascii_case("last"))
            {
                show_last_score(app);
            } else {
                start_task(
                    app,
                    task_tx,
                    TaskKind::Score {
                        url: optional_http_url(&parts, 1),
                    },
                );
            }
        }
        "/functional" => start_task(
            app,
            task_tx,
            TaskKind::Functional {
                pattern: parts.get(1).map(|s| (*s).to_string()),
            },
        ),
        "/lighthouse" => {
            let url = lighthouse_url(app, parts.get(1).copied());
            start_task(app, task_tx, TaskKind::Lighthouse { url });
        }
        "/playwright" => start_task(
            app,
            task_tx,
            TaskKind::Playwright {
                pattern: parts.get(1).map(|s| (*s).to_string()),
            },
        ),
        "/trivy" => start_task(app, task_tx, TaskKind::Trivy),
        "/arkenar" => {
            let url = lighthouse_url(app, parts.get(1).copied());
            start_task(app, task_tx, TaskKind::Arkenar { url });
        }
        "/upgrade" => push_json(app, &upgrade::check()),
        "/update" => push_json(app, &update::run_update(&app.workspace)),
        "/status" => push_json(app, &verify_progress::status_json(&app.workspace)),
        "/agents" => {
            app.push_blocks(
                FeedRole::System,
                vec![ContentBlock::code(app.agent_brief())],
            );
        }
        "/agent" => match parts.get(1).map(|s| s.to_lowercase()).as_deref() {
            Some("status") => push_json(app, &agent::status(&app.workspace)),
            Some("rules") => push_json(app, &agent::rules_json(&app.workspace)),
            Some("paths") => push_json(app, &agent::paths_json(&app.workspace)),
            Some("next-action") | Some("next") => {
                push_json(app, &agent::next_action_json(&app.workspace))
            }
            Some("plan") => push_json(app, &agent::plan(&app.workspace)),
            Some("handoff") => start_task(
                app,
                task_tx,
                TaskKind::Handoff {
                    params: parse_verify_flags(&parts),
                },
            ),
            _ => push_json(app, &agent::manifest(&app.workspace)),
        },
        "/stay-on-track" => match parts.get(1).map(|s| s.to_lowercase()).as_deref() {
            Some("disable") => match workspace::disable_stay_on_track_mode(&app.workspace) {
                Ok(()) => {
                    app.settings.stay_on_track_enabled = false;
                    config::save_settings(&app.settings);
                    app.push_system("Stay-on-track disabled for this workspace.");
                }
                Err(e) => app.push_system(&format!("Stay-on-track disable failed: {e}")),
            },
            Some("status") => {
                let ws = workspace::load_workspace_config(&app.workspace);
                app.push_system(&format!(
                    "Stay-on-track: {} (global default: {})",
                    if ws.stay_on_track { "enabled" } else { "disabled" },
                    if app.settings.stay_on_track_enabled {
                        "on"
                    } else {
                        "off"
                    }
                ));
            }
            _ => match workspace::enable_stay_on_track_mode(&app.workspace, &app.settings) {
                Ok(path) => {
                    app.settings.stay_on_track_enabled = true;
                    config::save_settings(&app.settings);
                    app.push_system(&format!(
                        "Stay-on-track enabled — skill at {}\nAgents read .playhouse/stay-on-track/PROJECT.md first.",
                        path.display()
                    ));
                }
                Err(e) => app.push_system(&format!("Stay-on-track failed: {e}")),
            },
        },
        "/skill" => match parts.get(1).map(|s| s.to_lowercase()).as_deref() {
            Some("disable") => match workspace::disable_playhouse_skill_mode(&app.workspace) {
                Ok(()) => {
                    app.settings.playhouse_skill_enabled = false;
                    config::save_settings(&app.settings);
                    app.push_system("Playhouse agent skill disabled for this workspace.");
                }
                Err(e) => app.push_system(&format!("Skill disable failed: {e}")),
            },
            Some("status") => {
                let path = workspace::playhouse_skill_path(&app.workspace, &app.settings);
                let exists = path.exists();
                app.push_system(&format!(
                    "Skill: {} · path: {} · global: {}",
                    if exists && app.settings.playhouse_skill_enabled {
                        "installed"
                    } else {
                        "not installed"
                    },
                    path.display(),
                    if app.settings.playhouse_skill_enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                ));
            }
            _ => match workspace::install_playhouse_skill(&app.workspace, &app.settings) {
                Ok(path) => {
                    app.settings.playhouse_skill_enabled = true;
                    config::save_settings(&app.settings);
                    app.push_system(&format!(
                        "Playhouse agent skill installed at {}\nRecommended: agents read .playhouse/SKILL.md first.",
                        path.display()
                    ));
                }
                Err(e) => app.push_system(&format!("Playhouse skill failed: {e}")),
            },
        },
        "/export" => match app.export_brief() {
            Ok(path) => app.push_system(&format!("Exported workspace brief to {}", path.display())),
            Err(e) => app.push_system(&format!("Export failed: {e}")),
        },
        "/config" | "/settings" => match parts.get(1).map(|s| s.to_lowercase()).as_deref() {
            Some("schema") => push_json(app, &config_cli::schema_json()),
            Some("get") if parts.len() >= 3 => match config_cli::get(&app.workspace, parts[2]) {
                Ok(v) => push_json(app, &v),
                Err(e) => app.push_system(&format!("Config get failed: {e}")),
            },
            Some("set") if parts.len() >= 4 => {
                let value = parts[3..].join(" ");
                match config_cli::set(&app.workspace, parts[2], &value) {
                    Ok(v) => {
                        push_json(app, &v);
                        app.invalidate_brief();
                        app.refresh_local_server();
                        app.refresh_config_options();
                    }
                    Err(e) => app.push_system(&format!("Config set failed: {e}")),
                }
            }
            _ => {
                app.mode = AppMode::ConfigMenu;
                app.config_tab = 0;
                app.config_selected = 0;
                app.refresh_config_options();
            }
        },
        "/help" => {
            app.mode = AppMode::HelpMenu;
            app.help_tab = 0;
            app.help_selected = 0;
        }
        "/clear" => {
            app.feed.clear();
            app.clear_task_feed();
            app.feed_stick_bottom = true;
            app.feed_scroll_pos = 0.0;
            app.feed_scroll_target = 0.0;
        }
        "/quit" | "/exit" => app.running = false,
        "/version" | "/v" => show_version(app),
        "/uninstall" => run_uninstall(app, &parts),
        "/auth" => match parts.get(1).map(|s| s.to_lowercase()).as_deref() {
            Some("login") => run_auth_login(app, &parts),
            _ => app.push_system(
                "Usage: /auth login --token TOKEN [--url URL] | --header-name N --header-value V | --basic-user U --basic-pass P",
            ),
        },
        "/test" => match parts.get(1).map(|s| s.to_lowercase()).as_deref() {
            Some("list") => {
                let profile = project::detect(&app.workspace);
                push_json(app, &baseplates::list_plates(&profile));
            }
            Some("init") => {
                let plate = parts
                    .iter()
                    .position(|p| *p == "--plate")
                    .and_then(|i| parts.get(i + 1).copied());
                let force = parts.contains(&"--force");
                match baseplates::init_plate(&app.workspace, plate, force) {
                    Ok(report) => push_json(app, &report),
                    Err(e) => app.push_system(&format!("Test init failed: {e}")),
                }
            }
            Some("add") => {
                let plate = parts
                    .iter()
                    .position(|p| *p == "--plate")
                    .and_then(|i| parts.get(i + 1).copied());
                let force = parts.contains(&"--force");
                match plate {
                    Some(id) => match baseplates::add_plate(&app.workspace, id, force) {
                        Ok(report) => push_json(app, &report),
                        Err(e) => app.push_system(&format!("Test add failed: {e}")),
                    },
                    None => app.push_system("Usage: /test add --plate PLATE_ID [--force]"),
                }
            }
            Some("run") => start_task(
                app,
                task_tx,
                TaskKind::Functional {
                    pattern: parts.get(2).map(|s| (*s).to_string()),
                },
            ),
            _ => app.push_system("Usage: /test list | init [--plate ID] [--force] | add --plate ID | run"),
        },
        other => app.push_system(&format!("Unknown command: {other}. Type /help")),
    }
}

/// Map common plain-text questions to slash commands (TUI-friendly).
fn natural_to_slash(text: &str) -> Option<String> {
    let normalized = text
        .trim()
        .trim_end_matches('?')
        .trim_end_matches('.')
        .to_lowercase();
    match normalized.as_str() {
        "version" | "what version" | "playhouse version" | "what's the version"
        | "whats the version" | "current version" => Some("/version".into()),
        "upgrade" | "check for updates" | "any updates" => Some("/upgrade".into()),
        "update" | "update playhouse" => Some("/update".into()),
        "help" | "commands" | "what can you do" => Some("/help".into()),
        "doctor" | "health" | "tool health" => Some("/doctor".into()),
        "verify" | "run verify" | "full verify" => Some("/verify".into()),
        "uninstall" | "remove tools" | "remove playhouse tools" => {
            Some("/uninstall".into())
        }
        "quit" | "exit" | "bye" => Some("/quit".into()),
        _ => None,
    }
}

fn show_version(app: &mut App) {
    let version = env!("CARGO_PKG_VERSION");
    let install = detect::run_doctor(&app.workspace)
        .into_iter()
        .find(|c| c.name == "Playhouse CLI")
        .map(|c| c.detail)
        .unwrap_or_else(|| "unknown".into());
    push_json(
        app,
        &serde_json::json!({
            "command": "version",
            "version": version,
            "install": install,
        }),
    );
}

fn run_uninstall(app: &mut App, parts: &[&str]) {
    let global = parts.contains(&"--global");
    let workspace_tools = parts.contains(&"--workspace-tools");
    let yes = parts.contains(&"--yes");
    if !yes {
        app.push_system(
            "Uninstall removes bundled Trivy/Arkenar and workspace npm tools.\n\
             Re-run: /uninstall --yes [--global] [--workspace-tools]",
        );
        return;
    }
    let remove_global = global || !workspace_tools;
    let remove_ws = workspace_tools || !global;
    let report = uninstall::uninstall_all(&app.workspace, remove_global, remove_ws);
    push_json(app, &report);
}

fn run_auth_login(app: &mut App, parts: &[&str]) {
    let flag = |name: &str| -> Option<String> {
        parts
            .iter()
            .position(|p| p.eq_ignore_ascii_case(name))
            .and_then(|i| parts.get(i + 1))
            .map(|s| (*s).to_string())
    };
    let headers = match auth::login_headers(
        flag("--token").as_deref(),
        flag("--header-name").as_deref(),
        flag("--header-value").as_deref(),
        flag("--basic-user").as_deref(),
        flag("--basic-pass").as_deref(),
    ) {
        Ok(h) => h,
        Err(e) => {
            app.push_system(&e);
            return;
        }
    };
    if let Some(url) = flag("--url") {
        if let Err(e) = auth::set_default_url_if_provided(&app.workspace, Some(&url)) {
            app.push_system(&format!("Config set failed: {e}"));
            return;
        }
    }
    match auth::save_auth_headers(&app.workspace, headers) {
        Ok(v) => push_json(app, &v),
        Err(e) => app.push_system(&format!("Auth login failed: {e}")),
    }
}

fn parse_verify_flags(parts: &[&str]) -> VerifyParams {
    let mut p = VerifyParams::new();
    let mut i = 1usize;
    if parts.len() > 1 && parts[1].eq_ignore_ascii_case("handoff") {
        i = 2;
    }
    while i < parts.len() {
        match parts[i] {
            "--test" if i + 1 < parts.len() => {
                p.test_pattern = Some(parts[i + 1].to_string());
                i += 2;
            }
            "--start-server" if i + 1 < parts.len() => {
                p.start_server = Some(parts[i + 1].to_string());
                i += 2;
            }
            "--port" | "--server-port" if i + 1 < parts.len() => {
                p.server_port = parts[i + 1].parse().ok();
                i += 2;
            }
            "--server-timeout" if i + 1 < parts.len() => {
                p.server_timeout = parts[i + 1].parse().unwrap_or(120);
                i += 2;
            }
            s if s.starts_with("http://") || s.starts_with("https://") => {
                p.url = Some(s.to_string());
                i += 1;
            }
            _ => i += 1,
        }
    }
    p
}

fn optional_http_url(parts: &[&str], from: usize) -> Option<String> {
    parts
        .iter()
        .skip(from)
        .find(|s| s.starts_with("http://") || s.starts_with("https://"))
        .map(|s| (*s).to_string())
}

fn show_last_score(app: &mut App) {
    match score::load_saved_report(&app.workspace) {
        Some((score, engines, exit_code)) => {
            app.push_blocks(
                FeedRole::System,
                vec![ContentBlock::score_report(score, exit_code, engines)],
            );
        }
        None => app.push_system("No saved score — run /verify or /score first."),
    }
}

fn lighthouse_url(app: &App, explicit: Option<&str>) -> String {
    explicit
        .map(String::from)
        .or_else(|| app.settings.default_lighthouse_url.clone())
        .or_else(|| crate::detect::find_local_server(&app.workspace))
        .unwrap_or_else(|| "http://localhost:3000".to_string())
}

fn push_json(app: &mut App, value: &impl serde::Serialize) {
    let text = serde_json::to_string_pretty(value).unwrap_or_default();
    app.push_blocks(FeedRole::System, vec![ContentBlock::code(text)]);
}

fn start_task(app: &mut App, task_tx: &mpsc::UnboundedSender<TaskEvent>, kind: TaskKind) {
    if app.is_busy() {
        app.push_system("Already running a task — wait for it to finish.");
        return;
    }
    app.push_user(&slash_label(&kind));
    spawn_task(kind, app.workspace.clone(), task_tx.clone());
}

pub fn slash_label(kind: &TaskKind) -> String {
    match kind {
        TaskKind::Doctor { resolve } => {
            if *resolve {
                "/doctor resolve".into()
            } else {
                "/doctor".into()
            }
        }
        TaskKind::Install => "/install".into(),
        TaskKind::Init { .. } => "/init".into(),
        TaskKind::Verify { params } => {
            if let Some(u) = &params.url {
                format!("/verify {u}")
            } else if params.start_server.is_some() || params.test_pattern.is_some() {
                "/verify (with options)".into()
            } else {
                "/verify".into()
            }
        }
        TaskKind::Score { url } => url
            .as_ref()
            .map(|u| format!("/score {u}"))
            .unwrap_or_else(|| "/score".into()),
        TaskKind::Handoff { params } => {
            if let Some(u) = &params.url {
                format!("/agent handoff {u}")
            } else {
                "/agent handoff".into()
            }
        }
        TaskKind::Lighthouse { url } => format!("/lighthouse {url}"),
        TaskKind::Playwright { .. } => "/playwright".into(),
        TaskKind::Functional { .. } => "/functional".into(),
        TaskKind::Trivy => "/trivy".into(),
        TaskKind::Arkenar { url } => format!("/arkenar {url}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slash_labels_match_commands() {
        assert_eq!(slash_label(&TaskKind::Doctor { resolve: false }), "/doctor");
        assert_eq!(slash_label(&TaskKind::Doctor { resolve: true }), "/doctor resolve");
        assert_eq!(slash_label(&TaskKind::Functional { pattern: None }), "/functional");
        assert_eq!(
            slash_label(&TaskKind::Handoff {
                params: VerifyParams::new()
            }),
            "/agent handoff"
        );
        assert_eq!(
            slash_label(&TaskKind::Verify {
                params: VerifyParams::new()
            }),
            "/verify"
        );
        assert_eq!(slash_label(&TaskKind::Score { url: None }), "/score");
        assert_eq!(
            slash_label(&TaskKind::Lighthouse {
                url: "http://localhost:3000".into()
            }),
            "/lighthouse http://localhost:3000"
        );
    }

    #[test]
    fn parse_verify_flags_extracts_url_and_options() {
        let parts: Vec<&str> = vec![
            "/verify",
            "http://localhost:3000",
            "--test",
            "login",
            "--start-server",
            "npm run dev",
            "--port",
            "3000",
            "--server-timeout",
            "90",
        ];
        let p = parse_verify_flags(&parts);
        assert_eq!(p.url.as_deref(), Some("http://localhost:3000"));
        assert_eq!(p.test_pattern.as_deref(), Some("login"));
        assert_eq!(p.start_server.as_deref(), Some("npm run dev"));
        assert_eq!(p.server_port, Some(3000));
        assert_eq!(p.server_timeout, 90);
    }

    #[test]
    fn parse_verify_flags_skips_handoff_prefix() {
        let parts: Vec<&str> = vec![
            "/agent",
            "handoff",
            "https://example.com",
            "--test",
            "smoke",
        ];
        let p = parse_verify_flags(&parts);
        assert_eq!(p.url.as_deref(), Some("https://example.com"));
        assert_eq!(p.test_pattern.as_deref(), Some("smoke"));
    }

    #[test]
    fn natural_to_slash_maps_version_questions() {
        assert_eq!(natural_to_slash("what version?"), Some("/version".into()));
        assert_eq!(natural_to_slash("help"), Some("/help".into()));
        assert_eq!(natural_to_slash("random note"), None);
    }
}
