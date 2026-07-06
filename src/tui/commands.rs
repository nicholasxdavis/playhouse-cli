use tokio::sync::mpsc;

use crate::detect;
use crate::tui::app::{App, AppMode, FeedRole, TaskKind};
use crate::tui::config;
use crate::tui::tasks::{spawn_task, TaskEvent};
use crate::tui::ui_blocks::ContentBlock;

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
    let cmd = command
        .split_whitespace()
        .next()
        .unwrap_or(command)
        .to_lowercase();
    match cmd.as_str() {
        "/doctor" => start_task(app, task_tx, TaskKind::Doctor),
        "/install" => start_task(app, task_tx, TaskKind::Install),
        "/init" => start_task(
            app,
            task_tx,
            TaskKind::Init {
                stay_on_track: app.settings.stay_on_track_enabled,
            },
        ),
        "/verify" | "/score" => start_task(app, task_tx, TaskKind::Verify),
        "/lighthouse" => {
            let url = app
                .settings
                .default_lighthouse_url
                .clone()
                .or_else(|| detect::find_local_server(&app.workspace))
                .unwrap_or_else(|| "http://localhost:3000".to_string());
            start_task(app, task_tx, TaskKind::Lighthouse { url });
        }
        "/playwright" => start_task(app, task_tx, TaskKind::Playwright { pattern: None }),
        "/trivy" => start_task(app, task_tx, TaskKind::Trivy),
        "/arkenar" => {
            let url = app
                .settings
                .default_lighthouse_url
                .clone()
                .or_else(|| detect::find_local_server(&app.workspace))
                .unwrap_or_else(|| "http://localhost:3000".to_string());
            start_task(app, task_tx, TaskKind::Arkenar { url });
        }
        "/agents" => {
            app.push_blocks(
                FeedRole::System,
                vec![ContentBlock::code(app.agent_brief())],
            );
        }
        "/agent" => {
            let manifest = crate::agent::manifest(&app.workspace);
            let text = serde_json::to_string_pretty(&manifest).unwrap_or_default();
            app.push_blocks(FeedRole::System, vec![ContentBlock::code(text)]);
            app.push_system("Headless: `playhouse agent --json`");
        }
        "/stay-on-track" => {
            match crate::workspace::enable_stay_on_track_mode(&app.workspace, &app.settings) {
                Ok(path) => {
                    app.settings.stay_on_track_enabled = true;
                    config::save_settings(&app.settings);
                    app.push_system(&format!(
                        "Stay-on-track enabled - skill at {}\nAgent reads .playhouse/stay-on-track/PROJECT.md first.",
                        path.display()
                    ));
                }
                Err(e) => app.push_system(&format!("Stay-on-track failed: {e}")),
            }
        }
        "/skill" => {
            match crate::workspace::install_playhouse_skill(&app.workspace, &app.settings) {
                Ok(path) => {
                    app.settings.playhouse_skill_enabled = true;
                    config::save_settings(&app.settings);
                    app.push_system(&format!(
                        "Playhouse agent skill installed at {}\nRecommended: agents read .playhouse/SKILL.md first.",
                        path.display()
                    ));
                }
                Err(e) => app.push_system(&format!("Playhouse skill failed: {e}")),
            }
        }
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
            app.feed_stick_bottom = true;
            app.feed_scroll_pos = 0.0;
            app.feed_scroll_target = 0.0;
        }
        "/quit" | "/exit" => app.running = false,
        other => app.push_system(&format!("Unknown command: {other}. Type /help")),
    }
}

fn start_task(app: &mut App, task_tx: &mpsc::UnboundedSender<TaskEvent>, kind: TaskKind) {
    if app.is_busy() {
        app.push_system("Already running a task - wait for it to finish.");
        return;
    }
    app.push_user(&slash_label(&kind));
    spawn_task(kind, app.workspace.clone(), task_tx.clone());
}

fn slash_label(kind: &TaskKind) -> String {
    match kind {
        TaskKind::Doctor => "/doctor".into(),
        TaskKind::Install => "/install".into(),
        TaskKind::Init { .. } => "/init".into(),
        TaskKind::Verify => "/verify".into(),
        TaskKind::Lighthouse { url } => format!("/lighthouse {url}"),
        TaskKind::Playwright { .. } => "/playwright".into(),
        TaskKind::Trivy => "/trivy".into(),
        TaskKind::Arkenar { url } => format!("/arkenar {url}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slash_labels_match_commands() {
        assert_eq!(slash_label(&TaskKind::Doctor), "/doctor");
        assert_eq!(
            slash_label(&TaskKind::Lighthouse {
                url: "http://localhost:3000".into()
            }),
            "/lighthouse http://localhost:3000"
        );
    }
}
