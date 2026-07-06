mod app;
mod ascii_asset;
mod clipboard;
mod commands;
mod components;
mod config;
mod keys;
mod layout;
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
mod workspace_status;

#[cfg(test)]
mod integration_tests;

use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Size;
use ratatui::Terminal;
use tokio::sync::mpsc;

pub use app::App;

use app::{AppMode, TaskKind};
use crate::workspace;
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

fn redraw(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    terminal.autoresize()?;
    terminal.draw(|f| ui::draw(f, app))?;
    Ok(())
}

/// Detect terminal size changes even when the backend misses a resize event (common on Windows).
fn sync_terminal_size(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    last: &mut Size,
) -> io::Result<bool> {
    terminal.autoresize()?;
    let size = terminal.size()?;
    if size == *last {
        return Ok(false);
    }
    *last = size;
    app.on_resize();
    Ok(true)
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
    terminal.autoresize()?;
    let mut last_size = terminal.size()?;

    theme::load_config();

    let first_run = splash::is_first_run(workspace);
    let showed_splash = if splash::skip_requested() {
        false
    } else {
        splash::run(&mut terminal, workspace, first_run)?;
        splash::mark_launched();
        true
    };

    let settings = config::load_settings();
    workspace::maybe_auto_init(workspace, &settings);

    let mut app = App::new(workspace, showed_splash);
    let (task_tx, mut task_rx) = mpsc::unbounded_channel();
    let tick_rate = Duration::from_millis(80);
    let mut needs_redraw = true;

    if app.settings.auto_run_doctor_on_start {
        spawn_task(
            TaskKind::Doctor { resolve: false },
            app.workspace.clone(),
            task_tx.clone(),
        );
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
                    for block in &mut blocks {
                        if let ContentBlock::ScoreReport { reveal_tick, .. } = block {
                            *reveal_tick = app.tick_count;
                        }
                    }
                    let already_shows_failure = blocks_show_failure(&blocks);
                    app.finish_task_feed(blocks);
                    if let Some((pass, total)) = doctor_stats {
                        app.set_doctor_stats(pass, total);
                    }
                    if !success && !already_shows_failure {
                        app.push_system(&format!("✗ {summary}"));
                    } else if success && app.settings.bell_enabled {
                        let _ = crossterm::execute!(io::stdout(), crossterm::style::Print("\x07"));
                    }
                    if app.settings.auto_export_agent_brief {
                        let _ = app.export_brief();
                    }
                    needs_redraw = true;
                }
            }
        }

        if sync_terminal_size(&mut terminal, &mut app, &mut last_size)? {
            needs_redraw = true;
        }

        // Refresh dev-server URL at most every 30s (never on resize — port probe blocks for seconds).
        if app.tick_count.is_multiple_of(375) && app.tick_count > 0 {
            app.refresh_local_server();
            needs_redraw = true;
        }

        if needs_redraw || app.needs_animation_frame() {
            redraw(&mut terminal, &mut app)?;
            needs_redraw = false;
        }

        if event::poll(tick_rate)? {
            loop {
                if !event::poll(Duration::from_millis(0))? {
                    break;
                }
                match event::read()? {
                    Event::Resize(_, _) => {
                        let _ = sync_terminal_size(&mut terminal, &mut app, &mut last_size);
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
                    Event::Mouse(m) => app.handle_mouse(m),
                    _ => {}
                }
            }
            // Redraw immediately after input so resize/key handling never leaves a blank frame.
            redraw(&mut terminal, &mut app)?;
            needs_redraw = false;
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

fn blocks_show_failure(blocks: &[ContentBlock]) -> bool {
    use ui_blocks::ToolStatus;
    blocks.iter().any(|b| match b {
        ContentBlock::ToolCall {
            status: ToolStatus::Error,
            ..
        } => true,
        ContentBlock::ScoreReport { score, .. } => !score.passed,
        ContentBlock::TodoList { items, .. } => items.iter().any(|i| {
            i.detail.as_ref().is_some_and(|d| {
                d.contains("failed") || d.contains("check failed") || d.contains("vulns")
            })
        }),
        _ => false,
    })
}
