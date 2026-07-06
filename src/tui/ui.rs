use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    widgets::Block,
    Frame,
};

use crate::tui::app::{App, AppMode};
use crate::tui::components;
use crate::tui::theme;

pub fn draw(f: &mut Frame, app: &mut App) {
    let bg = Block::default().style(Style::default().bg(theme::get_bg_color()));
    f.render_widget(bg, f.area());

    match app.mode {
        AppMode::HelpMenu => {
            components::help_menu::render(f, f.area(), app);
            return;
        }
        AppMode::ConfigMenu => {
            components::config_menu::render(f, f.area(), app);
            return;
        }
        _ => {}
    }

    let area = f.area();
    if area.width < 20 || area.height < 6 {
        components::status_bar::render(f, area, app);
        return;
    }

    let status_h = 1u16;
    let header_h = area.height.clamp(3, 6);
    let max_input = area
        .height
        .saturating_sub(header_h + status_h + 2)
        .clamp(3, 10);
    let input_height = components::input::block_height(app, area.width, area.height)
        .min(max_input)
        .max(3);

    let layout = Layout::vertical([
        Constraint::Length(header_h),
        Constraint::Min(1),
        Constraint::Length(input_height),
        Constraint::Length(status_h),
    ])
    .split(area);

    components::welcome::render(f, layout[0], app);
    components::feed::render(f, layout[1], app);
    components::input::render(f, layout[2], app);
    app.input_pane_area = layout[2];
    components::status_bar::render(f, layout[3], app);

    if app.mode == AppMode::SlashMenu {
        components::slash_menu::render(f, area, app);
    }
    if app.mode == AppMode::MentionMenu {
        components::mention_menu::render(f, area, app);
    }
}
