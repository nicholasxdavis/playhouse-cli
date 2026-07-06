use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::text_box;
use crate::tui::theme;

fn popup_area(area: Rect, height: u16, max_width: u16) -> Rect {
    let popup_x = area.x.saturating_add(1);
    let popup_width = area.width.saturating_sub(2).min(max_width).max(1);
    let right_limit = area.x.saturating_add(area.width);
    let width = popup_width.min(right_limit.saturating_sub(popup_x));
    let popup_y = if area.height > height.saturating_add(3) {
        area.y + area.height - height - 3
    } else {
        area.y
    };
    Rect {
        x: popup_x,
        y: popup_y,
        width,
        height,
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let filtered = app.filtered_slash_commands();
    if filtered.is_empty() {
        return;
    }

    let max_items = filtered.len().min(8);
    let popup_height = max_items as u16 + 1;
    let popup_area = popup_area(area, popup_height, 80);

    theme::clear_area(f, popup_area);

    let row_budget = popup_area.width as usize;
    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let is_selected = i == app.slash_selected;
            let marker = if is_selected { "› " } else { "  " };
            let cmd_style = if is_selected {
                theme::accent_bold()
            } else {
                theme::accent()
            };
            let desc_style = if is_selected {
                theme::text_bold()
            } else {
                theme::text_muted()
            };
            let cmd_part = format!("{marker}{}", cmd.command);
            let cmd_w = text_box::width(&cmd_part);
            let desc_budget = row_budget.saturating_sub(cmd_w + 1);
            let desc = text_box::truncate(&cmd.description, desc_budget);
            ListItem::new(Line::from(vec![
                Span::styled(marker, cmd_style),
                Span::styled(&cmd.command, cmd_style),
                Span::styled(" ", Style::default()),
                Span::styled(desc, desc_style),
            ]))
        })
        .collect();

    let list_area = Rect {
        x: popup_area.x,
        y: popup_area.y,
        width: popup_area.width,
        height: popup_area.height.saturating_sub(1),
    };
    let mut state = ListState::default();
    state.select(Some(app.slash_selected));
    f.render_widget(List::new(items), list_area);

    let footer_area = Rect {
        x: popup_area.x,
        y: popup_area.y + popup_area.height.saturating_sub(1),
        width: popup_area.width,
        height: 1,
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            text_box::truncate(
                "  filter · ↑/↓ navigate · Enter select · Esc cancel",
                popup_area.width as usize,
            ),
            theme::text_dim(),
        ))),
        footer_area,
    );
}
