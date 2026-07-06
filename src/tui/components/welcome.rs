use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::spinner;
use crate::tui::theme;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_focused());

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let pass = app.tools_summary.clone();
    let task_label = if app.is_busy() {
        let spin = spinner::frame(app.tick_count);
        format!("{spin} {}", app.busy_label)
    } else {
        "ready".to_string()
    };
    let task_style = if app.is_busy() {
        theme::status_busy()
    } else {
        theme::status_ready()
    };

    let server = app
        .local_server
        .as_deref()
        .unwrap_or("no dev server");

    let line1 = Line::from(vec![
        Span::styled("Playhouse", theme::accent_bold()),
        Span::styled(" · ", theme::text_dim()),
        Span::styled(task_label, task_style),
        Span::styled(" · ", theme::text_dim()),
        Span::styled(pass, theme::text()),
    ]);

    let line2 = Line::from(vec![
        Span::styled("  ", theme::text_dim()),
        Span::styled(truncate_path(&app.workspace, inner.width as usize / 2), theme::text_muted()),
        Span::styled(" · ", theme::text_dim()),
        Span::styled(server, theme::accent()),
    ]);

    let avail = inner.height as usize;
    if avail >= 2 {
        f.render_widget(Paragraph::new(line1), Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        });
        f.render_widget(Paragraph::new(line2), Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: 1,
        });
    } else {
        f.render_widget(Paragraph::new(line1), inner);
    }
}

fn truncate_path(path: &str, max: usize) -> String {
    if path.len() <= max {
        path.to_string()
    } else {
        format!("…{}", &path[path.len().saturating_sub(max.saturating_sub(1))..])
    }
}
