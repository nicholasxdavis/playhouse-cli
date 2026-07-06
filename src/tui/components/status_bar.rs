use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::App;
use crate::tui::spinner;
use crate::tui::text_box;
use crate::tui::theme;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let busy = app.is_busy();
    let status_label = if busy {
        let spin = spinner::frame(app.tick_count);
        format!(" {spin} {}", app.busy_label)
    } else {
        " ready".to_string()
    };
    let status_style = if busy {
        theme::status_busy()
    } else {
        theme::status_ready()
    };

    let mut spans = vec![
        Span::styled(status_label, status_style),
        Span::styled(" | ", theme::text_dim()),
        Span::styled(
            text_box::truncate(&app.workspace, area.width as usize / 2),
            theme::text_muted(),
        ),
    ];

    if app.feed_scrolled_up() {
        let from_bottom = app
            .feed_max_scroll_top()
            .saturating_sub(app.feed_scroll_line());
        spans.push(Span::styled(" | ", theme::text_dim()));
        spans.push(Span::styled(
            format!("feed up {from_bottom}"),
            theme::text_muted(),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
