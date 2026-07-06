use crossterm::event::{KeyCode, KeyModifiers};
use tokio::sync::mpsc;

use crate::tui::app::{App, AppMode, FeedRole};
use crate::tui::commands::{execute_command, submit_input};
use crate::tui::keys::{
    handle_config_key, handle_help_key, handle_normal_key, handle_slash_key, is_copy_shortcut,
    is_paste_shortcut,
};
use crate::tui::tasks::TaskEvent;

fn temp_workspace() -> String {
    let dir = std::env::temp_dir().join(format!("playhouse-tui-it-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.to_string_lossy().into_owned()
}

fn test_app() -> App {
    App::new(&temp_workspace(), false)
}

fn noop_task_tx() -> mpsc::UnboundedSender<TaskEvent> {
    let (tx, _rx) = mpsc::unbounded_channel();
    tx
}

fn last_system_text(app: &App) -> Option<String> {
    app.feed.iter().rev().find_map(|entry| {
        if entry.role != FeedRole::System {
            return None;
        }
        entry.blocks.first().and_then(|b| match b {
            crate::tui::ui_blocks::ContentBlock::Text { content } => Some(content.clone()),
            _ => None,
        })
    })
}

#[test]
fn keyboard_shortcuts() {
    assert!(is_copy_shortcut(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL
    ));
    assert!(is_paste_shortcut(KeyCode::Char('p'), KeyModifiers::CONTROL));
    assert!(is_paste_shortcut(KeyCode::Char('v'), KeyModifiers::CONTROL));
    assert!(is_paste_shortcut(KeyCode::Insert, KeyModifiers::SHIFT));
}

#[test]
fn normal_key_opens_slash_menu() {
    let mut app = test_app();
    let tx = noop_task_tx();
    handle_normal_key(&mut app, KeyCode::Char('/'), KeyModifiers::empty(), &tx);
    assert_eq!(app.mode, AppMode::SlashMenu);
    assert_eq!(app.input_text, "/");
}

#[test]
fn normal_key_escape_clears_input() {
    let mut app = test_app();
    app.input_text = "draft".into();
    app.cursor_position = 5;
    let tx = noop_task_tx();
    handle_normal_key(&mut app, KeyCode::Esc, KeyModifiers::empty(), &tx);
    assert!(app.input_text.is_empty());
    assert_eq!(app.cursor_position, 0);
}

#[test]
fn slash_menu_help_sets_mode() {
    let mut app = test_app();
    let tx = noop_task_tx();
    app.mode = AppMode::SlashMenu;
    app.input_text = "/help".into();
    app.slash_filter = "help".into();
    handle_slash_key(&mut app, KeyCode::Enter, KeyModifiers::empty(), &tx);
    assert_eq!(app.mode, AppMode::HelpMenu);
}

#[test]
fn execute_help_command() {
    let mut app = test_app();
    let tx = noop_task_tx();
    execute_command(&mut app, "/help", &tx);
    assert_eq!(app.mode, AppMode::HelpMenu);
}

#[test]
fn execute_unknown_command_shows_hint() {
    let mut app = test_app();
    let tx = noop_task_tx();
    execute_command(&mut app, "/not-a-real-cmd", &tx);
    let msg = last_system_text(&app).unwrap_or_default();
    assert!(msg.contains("Unknown command"));
}

#[test]
fn execute_clear_empties_feed() {
    let mut app = test_app();
    app.push_system("noise");
    let tx = noop_task_tx();
    execute_command(&mut app, "/clear", &tx);
    assert!(app.feed.is_empty());
}

#[test]
fn submit_advisory_persists_note() {
    let mut app = test_app();
    app.input_text = "ship checklist".into();
    let tx = noop_task_tx();
    submit_input(&mut app, &tx);
    assert!(app.advisories.iter().any(|n| n.contains("ship checklist")));
}

#[test]
fn config_menu_escape_saves() {
    let mut app = test_app();
    app.mode = AppMode::ConfigMenu;
    handle_config_key(&mut app, KeyCode::Esc);
    assert_eq!(app.mode, AppMode::Normal);
}

#[test]
fn help_menu_escape_returns_normal() {
    let mut app = test_app();
    app.mode = AppMode::HelpMenu;
    let tx = noop_task_tx();
    handle_help_key(&mut app, KeyCode::Esc, KeyModifiers::empty(), &tx);
    assert_eq!(app.mode, AppMode::Normal);
}

#[test]
fn slash_commands_include_v030_wiring() {
    let app = test_app();
    let cmds: Vec<&str> = app.slash_commands.iter().map(|c| c.command.as_str()).collect();
    for expected in ["/functional", "/status", "/upgrade", "/update", "/agent"] {
        assert!(cmds.contains(&expected), "missing slash command {expected}");
    }
}

fn feed_json_contains(app: &App, needle: &str) -> bool {
    app.feed.iter().any(|entry| {
        entry.role == FeedRole::System
            && entry
                .blocks
                .iter()
                .any(|b| matches!(b, crate::tui::ui_blocks::ContentBlock::Code { content } if content.contains(needle)))
    })
}

#[test]
fn execute_upgrade_emits_json() {
    let mut app = test_app();
    let tx = noop_task_tx();
    execute_command(&mut app, "/upgrade", &tx);
    assert!(feed_json_contains(&app, "\"current\""));
}

#[test]
fn execute_status_emits_json() {
    let mut app = test_app();
    let tx = noop_task_tx();
    execute_command(&mut app, "/status", &tx);
    assert!(feed_json_contains(&app, "\"command\""));
}

#[test]
fn execute_agent_status_emits_json() {
    let mut app = test_app();
    let tx = noop_task_tx();
    execute_command(&mut app, "/agent status", &tx);
    assert!(feed_json_contains(&app, "nextActions") || feed_json_contains(&app, "workspace"));
}

#[tokio::test]
async fn execute_functional_queues_user_command() {
    let mut app = test_app();
    let (tx, mut rx) = mpsc::unbounded_channel();
    execute_command(&mut app, "/functional", &tx);
    assert!(app.feed.iter().any(|e| e.role == FeedRole::User));
    let event = tokio::time::timeout(std::time::Duration::from_secs(10), rx.recv())
        .await
        .expect("functional task did not start in time")
        .expect("task channel closed");
    assert!(matches!(event, TaskEvent::Started { .. }));
}
