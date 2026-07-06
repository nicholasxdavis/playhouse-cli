use crossterm::event::{KeyCode, KeyModifiers};
use tokio::sync::mpsc;

use crate::tui::app::{App, AppMode};
use crate::tui::commands::{execute_command, submit_input};
use crate::tui::tasks::TaskEvent;

pub fn is_copy_shortcut(code: KeyCode, mods: KeyModifiers) -> bool {
    (code == KeyCode::Char('c')
        && mods.contains(KeyModifiers::CONTROL)
        && mods.contains(KeyModifiers::SHIFT))
        || (code == KeyCode::Insert && mods.contains(KeyModifiers::CONTROL))
}

pub fn is_paste_shortcut(code: KeyCode, mods: KeyModifiers) -> bool {
    (code == KeyCode::Char('v') && mods.contains(KeyModifiers::CONTROL))
        || (code == KeyCode::Insert && mods.contains(KeyModifiers::SHIFT))
}

pub fn handle_normal_key(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    task_tx: &mpsc::UnboundedSender<TaskEvent>,
) {
    let input_empty = app.input_text.is_empty();
    match code {
        KeyCode::Char('c')
            if modifiers.contains(KeyModifiers::CONTROL)
                && !modifiers.contains(KeyModifiers::SHIFT) =>
        {
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

pub fn handle_slash_key(
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

pub fn handle_mention_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
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

pub fn handle_help_key(
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
        KeyCode::Down if app.help_tab == 0 && app.help_selected + 1 < app.slash_commands.len() => {
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

pub fn handle_config_key(app: &mut App, code: KeyCode) {
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
