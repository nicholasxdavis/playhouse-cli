use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::text_box;
use crate::tui::theme;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let border_style = if matches!(
        app.mode,
        crate::tui::app::AppMode::Normal | crate::tui::app::AppMode::MentionMenu
    ) {
        theme::border_focused()
    } else {
        theme::border()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" Commands / notes ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    let inner_width = inner.width.saturating_sub(2).max(1) as usize;
    let input_text = &app.input_text;
    let cursor_pos = app.cursor_position;

    if input_text.is_empty() {
        let placeholder = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(
                    if app.is_busy() {
                        "Running QA task…"
                    } else {
                        "/doctor · /verify · @ files · Enter for notes…"
                    },
                    theme::text_dim(),
                ),
            ]),
            Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(
                    "Enter send · Shift+Enter newline · Tab complete",
                    theme::text_dim(),
                ),
            ]),
        ]);
        f.render_widget(placeholder, inner);
        return;
    }

    let rows = text_box::input_visual_layout(input_text, inner_width);
    let selection = app.input_selection();
    let mut lines: Vec<Line> = Vec::new();

    for (row_start, row_text) in &rows {
        let row_end = row_start + row_text.chars().count();
        let mut spans: Vec<Span> = vec![Span::styled(" ", Style::default())];
        for (col, ch) in row_text.chars().enumerate() {
            let global = row_start + col;
            let is_cursor = global == cursor_pos;
            let in_sel = selection
                .map(|s| global >= s.start && global < s.end)
                .unwrap_or(false);
            let style = if is_cursor {
                Style::default()
                    .fg(theme::get_bg_color())
                    .bg(theme::get_text_primary())
                    .add_modifier(Modifier::BOLD)
            } else if in_sel {
                Style::default()
                    .fg(theme::get_bg_color())
                    .bg(theme::get_text_primary())
            } else {
                theme::text()
            };
            spans.push(Span::styled(ch.to_string(), style));
        }
        if cursor_pos == row_end && cursor_pos == input_text.chars().count() {
            spans.push(Span::styled(
                " ",
                Style::default()
                    .fg(theme::get_bg_color())
                    .bg(theme::get_text_primary())
                    .add_modifier(Modifier::BOLD),
            ));
        }
        lines.push(Line::from(spans));
    }

    if rows.is_empty() {
        lines.push(Line::from(Span::styled(
            " ",
            Style::default()
                .fg(theme::get_bg_color())
                .bg(theme::get_text_primary())
                .add_modifier(Modifier::BOLD),
        )));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

pub fn block_height(app: &App, pane_width: u16, terminal_height: u16) -> u16 {
    const MIN: u16 = 3;
    let max = terminal_height.saturating_sub(6).clamp(MIN, 8);
    if app.input_text.is_empty() {
        return 5.min(max);
    }
    let inner_width = pane_width.saturating_sub(4).max(1) as usize;
    let rows = text_box::input_visual_layout(&app.input_text, inner_width);
    (rows.len() as u16 + 2).clamp(5, max)
}
