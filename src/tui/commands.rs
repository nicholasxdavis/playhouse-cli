use tokio::sync::mpsc;

use crate::agent;
use crate::score;
use crate::tui::app::{App, AppMode, FeedRole, TaskKind};
use crate::tui::config;
use crate::tui::tasks::{spawn_task, TaskEvent};
use crate::tui::ui_blocks::ContentBlock;
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
                url: optional_http_url(&parts, 1),
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
                    url: optional_http_url(&parts, 2).or_else(|| optional_http_url(&parts, 1)),
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
        "/config" | "/settings" => {
            app.mode = AppMode::ConfigMenu;
            app.config_tab = 0;
            app.config_selected = 0;
            app.refresh_config_options();
        }
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
        other => app.push_system(&format!("Unknown command: {other}. Type /help")),
    }
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
        TaskKind::Verify { url } => url
            .as_ref()
            .map(|u| format!("/verify {u}"))
            .unwrap_or_else(|| "/verify".into()),
        TaskKind::Score { url } => url
            .as_ref()
            .map(|u| format!("/score {u}"))
            .unwrap_or_else(|| "/score".into()),
        TaskKind::Handoff { url } => url
            .as_ref()
            .map(|u| format!("/agent handoff {u}"))
            .unwrap_or_else(|| "/agent handoff".into()),
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
            slash_label(&TaskKind::Handoff { url: None }),
            "/agent handoff"
        );
        assert_eq!(slash_label(&TaskKind::Score { url: None }), "/score");
        assert_eq!(
            slash_label(&TaskKind::Lighthouse {
                url: "http://localhost:3000".into()
            }),
            "/lighthouse http://localhost:3000"
        );
    }
}
