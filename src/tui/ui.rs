use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Clear},
    Frame,
};

use crate::tui::app::{App, AppMode};
use crate::tui::components;
use crate::tui::layout::{self, MainPanels};
use crate::tui::theme;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    if area.width == 0 || area.height == 0 {
        return;
    }

    // Full clear prevents resize ghosting on Windows terminals.
    f.render_widget(Clear, area);
    f.render_widget(
        Block::default().style(Style::default().bg(theme::get_bg_color())),
        area,
    );

    match app.mode {
        AppMode::HelpMenu => {
            components::help_menu::render(f, area, app);
            return;
        }
        AppMode::ConfigMenu => {
            components::config_menu::render(f, area, app);
            return;
        }
        _ => {}
    }

    if layout::is_tiny(area) {
        components::status_bar::render(f, area, app);
        app.input_pane_area = Rect::default();
        app.feed_pane_area = Rect::default();
        return;
    }

    let panels = layout::split_main(area, app);
    render_panels(f, app, panels);
}

fn render_panels(f: &mut Frame, app: &mut App, panels: MainPanels) {
    if panels.header.height > 0 && panels.header.width > 0 {
        if layout::is_compact(f.area()) {
            components::welcome::render_compact(f, panels.header, app);
        } else {
            components::welcome::render(f, panels.header, app);
        }
    }

    if panels.feed.height > 0 && panels.feed.width > 0 {
        components::feed::render(f, panels.feed, app);
    } else {
        app.feed_pane_area = Rect::default();
    }

    if panels.input.height > 0 && panels.input.width > 0 {
        components::input::render(f, panels.input, app);
        app.input_pane_area = panels.input;
    } else {
        app.input_pane_area = Rect::default();
    }

    if panels.status.height > 0 {
        components::status_bar::render(f, panels.status, app);
    }

    let area = f.area();
    if app.mode == AppMode::SlashMenu {
        components::slash_menu::render(f, area, app);
    }
    if app.mode == AppMode::MentionMenu {
        components::mention_menu::render(f, area, app);
    }
}
