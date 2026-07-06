mod app;
mod clipboard;
mod components;
mod config;
mod mention;
mod selection;
mod spinner;
mod tasks;
mod text_box;
mod theme;
pub use theme::{set_accent_from_string, set_light_theme};
mod ui;
mod ui_blocks;
mod walk;

use std::io;
use std::time::Duration;

use crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEventKind,
    KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;

pub use app::App;

use app::{AppMode, FeedRole, TaskKind};
use tasks::{spawn_task, TaskEvent};
use ui_blocks::ContentBlock;

use crate::detect;

pub async fn run(workspace: &str) -> i32 {
    if let Err(e) = run_inner(workspace).await {
        eprintln!("TUI error: {e}");
        return 1;
    }
    0
}

async fn run_inner(workspace: &str) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableBracketedPaste,
        event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    theme::load_config();
    let mut app = App::new(workspace);
    let (task_tx, mut task_rx) = mpsc::unbounded_channel();
    let tick_rate = Duration::from_millis(80);
    let mut needs_redraw = true;
    let mut resized = false;

    if app.settings.auto_run_doctor_on_start {
        spawn_task(TaskKind::Doctor, app.workspace.clone(), task_tx.clone());
    } else if app.settings.auto_install_tools {
        spawn_task(TaskKind::Install, app.workspace.clone(), task_tx.clone());
    }

    loop {
        app.tick();

        while let Ok(event) = task_rx.try_recv() {
            match event {
                TaskEvent::Started { label } => {
                    app.busy = true;
                    app.busy_label = label.clone();
                    app.clear_task_feed();
                    if !label.starts_with("Verify") {
                        app.push_blocks(
                            FeedRole::System,
                            vec![ContentBlock::tool_running("QA", &label)],
                        );
                    }
                    needs_redraw = true;
                }
                TaskEvent::Progress { label, blocks } => {
                    app.busy_label = label;
                    app.update_last_system_blocks(blocks);
                    needs_redraw = true;
                }
                TaskEvent::Finished {
                    mut blocks,
                    success,
                    summary,
                    doctor_stats,
                } => {
                    app.busy = false;
                    app.busy_label.clear();
                    app.clear_task_feed();
                    for block in &mut blocks {
                        if let ContentBlock::ScoreReport { reveal_tick, .. } = block {
                            *reveal_tick = app.tick_count;
                        }
                    }
                    app.push_blocks(FeedRole::System, blocks);
                    if let Some((pass, total)) = doctor_stats {
                        app.set_doctor_stats(pass, total);
                    }
                    if !success {
                        app.push_system(&format!("✗ {summary}"));
                    } else if app.settings.bell_enabled {
                        let _ = crossterm::execute!(io::stdout(), crossterm::style::Print("\x07"));
                    }
                    if app.settings.auto_export_agent_brief {
                        let _ = app.export_brief();
                    }
                    needs_redraw = true;
                }
            }
        }

        if needs_redraw || app.needs_animation_frame() {
            if resized {
                terminal.autoresize()?;
                resized = false;
            }
            terminal.draw(|f| ui::draw(f, &mut app))?;
            needs_redraw = false;
        }

        if event::poll(tick_rate)? {
            needs_redraw = true;
            match event::read()? {
                Event::Resize(_w, _h) => {
                    app.on_resize();
                    resized = true;
                }
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if is_copy_shortcut(key.code, key.modifiers) {
                        let _ = app.copy_selection();
                    } else if is_paste_shortcut(key.code, key.modifiers) {
                        if let Some(text) = clipboard::read_text() {
                            app.handle_paste(&text);
                        }
                    } else {
                        match app.mode {
                            AppMode::Normal => handle_normal_key(&mut app, key.code, key.modifiers, &task_tx),
                            AppMode::SlashMenu => {
                                handle_slash_key(&mut app, key.code, key.modifiers, &task_tx)
                            }
                            AppMode::MentionMenu => {
                                handle_mention_key(&mut app, key.code, key.modifiers)
                            }
                            AppMode::HelpMenu => handle_help_key(&mut app, key.code, key.modifiers, &task_tx),
                            AppMode::ConfigMenu => handle_config_key(&mut app, key.code),
                        }
                    }
                }
                Event::Paste(text) => app.handle_paste(&text),
                Event::Mouse(m) if app.mode == AppMode::Normal => {
                    app.handle_feed_mouse(m);
                }
                _ => {}
            }
        }

        if !app.running {
            break;
        }
    }

    app.save_settings();
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableBracketedPaste,
        event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    println!("\n  Playhouse - See you next time.\n");
    Ok(())
}

fn is_copy_shortcut(code: KeyCode, mods: KeyModifiers) -> bool {
    (code == KeyCode::Char('c')
        && mods.contains(KeyModifiers::CONTROL)
        && mods.contains(KeyModifiers::SHIFT))
        || (code == KeyCode::Insert && mods.contains(KeyModifiers::CONTROL))
}

fn is_paste_shortcut(code: KeyCode, mods: KeyModifiers) -> bool {
    (code == KeyCode::Char('v') && mods.contains(KeyModifiers::CONTROL))
        || (code == KeyCode::Insert && mods.contains(KeyModifiers::SHIFT))
}

fn handle_normal_key(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    task_tx: &mpsc::UnboundedSender<TaskEvent>,
) {
    let input_empty = app.input_text.is_empty();
    match code {
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) && !modifiers.contains(KeyModifiers::SHIFT) => {
            if app.copy_selection() {
                app.push_system("Copied selection");
            } else if app.register_ctrl_c() {
                app.running = false;
            } else {
                let left = 3_u8.saturating_sub(app.ctrl_c_streak);
                app.push_system(&format!(
                    "Press Ctrl+C {left} more time(s) to quit · Ctrl+Shift+C copies"
                ));
            }
        }
        KeyCode::Char('a') if modifiers.contains(KeyModifiers::CONTROL) => app.select_all_input(),
        KeyCode::Esc => {
            app.input_text.clear();
            app.cursor_position = 0;
            app.clear_text_selection();
        }
        KeyCode::Enter if modifiers.contains(KeyModifiers::SHIFT) => app.insert_char('\n'),
        KeyCode::Enter => submit_input(app, task_tx),
        KeyCode::Backspace => app.delete_char(),
        KeyCode::Delete => app.delete_forward(),
        KeyCode::Left if modifiers.contains(KeyModifiers::SHIFT) => app.move_cursor_left(true),
        KeyCode::Right if modifiers.contains(KeyModifiers::SHIFT) => app.move_cursor_right(true),
        KeyCode::Left => app.move_cursor_left(false),
        KeyCode::Right => app.move_cursor_right(false),
        KeyCode::Home => {
            app.cursor_position = 0;
            if !modifiers.contains(KeyModifiers::SHIFT) {
                app.input_select_anchor = None;
            }
        }
        KeyCode::End => {
            app.cursor_position = app.input_text.chars().count();
            if !modifiers.contains(KeyModifiers::SHIFT) {
                app.input_select_anchor = None;
            }
        }
        KeyCode::Up if input_empty => app.scroll_feed_up(1),
        KeyCode::Down if input_empty => app.scroll_feed_down(1),
        KeyCode::Up if modifiers.contains(KeyModifiers::SHIFT) => app.scroll_feed_up(3),
        KeyCode::Down if modifiers.contains(KeyModifiers::SHIFT) => app.scroll_feed_down(3),
        KeyCode::PageUp if input_empty => app.scroll_feed_page_up(),
        KeyCode::PageDown if input_empty => app.scroll_feed_page_down(),
        KeyCode::Char('g') if modifiers.contains(KeyModifiers::CONTROL) => app.scroll_feed_top(),
        KeyCode::Char('G') if modifiers.contains(KeyModifiers::CONTROL) => app.scroll_feed_bottom(),
        KeyCode::Char('/') => {
            app.mode = AppMode::SlashMenu;
            app.slash_filter.clear();
            app.slash_selected = 0;
            if !app.input_text.starts_with('/') {
                app.input_text = "/".to_string();
                app.cursor_position = 1;
            }
        }
        KeyCode::Char(ch) if !modifiers.contains(KeyModifiers::CONTROL) => app.insert_char(ch),
        _ => {}
    }
}

fn handle_slash_key(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    task_tx: &mpsc::UnboundedSender<TaskEvent>,
) {
    match code {
        KeyCode::Esc => {
            app.mode = AppMode::Normal;
            app.input_text.clear();
            app.cursor_position = 0;
            app.slash_filter.clear();
        }
        KeyCode::Enter => {
            let filtered = app.filtered_slash_commands();
            if let Some(cmd) = filtered.get(app.slash_selected) {
                let command = cmd.command.clone();
                app.mode = AppMode::Normal;
                app.input_text.clear();
                app.cursor_position = 0;
                app.slash_filter.clear();
                execute_command(app, &command, task_tx);
            }
        }
        KeyCode::Up if app.slash_selected > 0 => app.slash_selected -= 1,
        KeyCode::Down => {
            let max = app.filtered_slash_commands().len().saturating_sub(1);
            if app.slash_selected < max {
                app.slash_selected += 1;
            }
        }
        KeyCode::Tab => {
            let filtered = app.filtered_slash_commands();
            if let Some(cmd) = filtered.get(app.slash_selected) {
                app.input_text = cmd.command.clone();
                app.cursor_position = app.input_text.chars().count();
                app.slash_filter = app.input_text.trim_start_matches('/').to_string();
            }
        }
        KeyCode::Backspace => {
            if app.input_text.len() <= 1 {
                app.mode = AppMode::Normal;
                app.input_text.clear();
                app.cursor_position = 0;
            } else {
                app.delete_char();
                app.slash_filter = app.input_text.trim_start_matches('/').to_string();
                app.slash_selected = 0;
            }
        }
        KeyCode::Char(ch) if !modifiers.contains(KeyModifiers::CONTROL) => {
            app.insert_char(ch);
            app.slash_filter = app.input_text.trim_start_matches('/').to_string();
            app.slash_selected = 0;
        }
        _ => {}
    }
}

fn handle_mention_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        KeyCode::Esc => {
            app.mode = AppMode::Normal;
            app.mention_filtered.clear();
        }
        KeyCode::Enter if modifiers.contains(KeyModifiers::SHIFT) => app.insert_char('\n'),
        KeyCode::Enter => app.mention_menu_select(),
        KeyCode::Tab => app.mention_menu_complete(),
        KeyCode::Up => app.mention_menu_up(),
        KeyCode::Down => app.mention_menu_down(),
        KeyCode::Backspace => app.delete_char(),
        KeyCode::Char(ch) if !modifiers.contains(KeyModifiers::CONTROL) => app.insert_char(ch),
        _ => {}
    }
}

fn handle_help_key(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    task_tx: &mpsc::UnboundedSender<TaskEvent>,
) {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.mode = AppMode::Normal,
        KeyCode::Left | KeyCode::BackTab => {
            if app.help_tab > 0 {
                app.help_tab -= 1;
            } else {
                app.help_tab = 3;
            }
            app.help_selected = 0;
        }
        KeyCode::Right | KeyCode::Tab => {
            app.help_tab = (app.help_tab + 1) % 4;
            app.help_selected = 0;
        }
        KeyCode::Up if app.help_tab == 0 && app.help_selected > 0 => app.help_selected -= 1,
        KeyCode::Down if app.help_tab == 0
            && app.help_selected + 1 < app.slash_commands.len() =>
        {
            app.help_selected += 1;
        }
        KeyCode::Enter if app.help_tab == 0 => {
            if let Some(cmd) = app.slash_commands.get(app.help_selected) {
                let command = cmd.command.clone();
                app.mode = AppMode::Normal;
                execute_command(app, &command, task_tx);
            }
        }
        _ if modifiers.contains(KeyModifiers::CONTROL) => {}
        _ => {}
    }
}

fn handle_config_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = AppMode::Normal;
            app.save_settings();
        }
        KeyCode::Left | KeyCode::BackTab => {
            if app.config_tab > 0 {
                app.config_tab -= 1;
            } else {
                app.config_tab = 4;
            }
            app.config_selected = 0;
            app.refresh_config_options();
        }
        KeyCode::Right | KeyCode::Tab => {
            app.config_tab = (app.config_tab + 1) % 5;
            app.config_selected = 0;
            app.refresh_config_options();
        }
        KeyCode::Up if app.config_selected > 0 => app.config_selected -= 1,
        KeyCode::Down if app.config_selected + 1 < app.config_options.len() => {
            app.config_selected += 1;
        }
        KeyCode::Enter | KeyCode::Char(' ') => app.toggle_config(),
        _ => {}
    }
}

fn submit_input(app: &mut App, task_tx: &mpsc::UnboundedSender<TaskEvent>) {
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

fn execute_command(app: &mut App, command: &str, task_tx: &mpsc::UnboundedSender<TaskEvent>) {
    let cmd = command.split_whitespace().next().unwrap_or(command).to_lowercase();
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
