mod app;
mod clipboard;
mod commands;
mod components;
mod config;
mod keys;
mod mascot;
mod mention;
mod selection;
mod spinner;
mod splash;
mod tasks;
mod text_box;
mod theme;
pub use theme::{set_accent_from_string, set_light_theme};
mod ui;
mod ui_blocks;
mod walk;

#[cfg(test)]
mod integration_tests;

use std::io;
use std::time::Duration;

use crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyEventKind,
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
use keys::{handle_config_key, handle_help_key, handle_mention_key, handle_normal_key,
           handle_slash_key, is_copy_shortcut, is_paste_shortcut};
use tasks::{spawn_task, TaskEvent};
use ui_blocks::ContentBlock;

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

    let first_run = splash::is_first_run(workspace);
    let showed_splash = if splash::skip_requested() {
        false
    } else {
        splash::run(&mut terminal, workspace, first_run)?;
        splash::mark_launched();
        true
    };

    let mut app = App::new(workspace, showed_splash);
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
                            AppMode::Normal => {
                                handle_normal_key(&mut app, key.code, key.modifiers, &task_tx)
                            }
                            AppMode::SlashMenu => {
                                handle_slash_key(&mut app, key.code, key.modifiers, &task_tx)
                            }
                            AppMode::MentionMenu => {
                                handle_mention_key(&mut app, key.code, key.modifiers)
                            }
                            AppMode::HelpMenu => {
                                handle_help_key(&mut app, key.code, key.modifiers, &task_tx)
                            }
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
    println!("\n  Thanks for using Playhouse. Run `playhouse` anytime.\n");
    Ok(())
}
