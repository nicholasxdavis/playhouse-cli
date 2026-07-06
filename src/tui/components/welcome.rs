use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::mascot;
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

    let art_lines = mascot::load_lines(&app.workspace);
    let art_width = art_lines
        .iter()
        .map(|line| unicode_width::UnicodeWidthStr::width(line.as_str()))
        .max()
        .unwrap_or(18)
        .clamp(18, 28) as u16;

    const MASCOT_LEFT_PAD: u16 = 1;
    let cols = Layout::horizontal([
        Constraint::Length(art_width + MASCOT_LEFT_PAD + 2),
        Constraint::Min(24),
    ])
    .split(inner);

    let mascot_cols = Layout::horizontal([
        Constraint::Length(MASCOT_LEFT_PAD),
        Constraint::Min(art_width),
    ])
    .split(cols[0]);

    let box_height = mascot_cols[1].height as usize;
    let top_pad = box_height.saturating_sub(art_lines.len()) / 2;
    let mut char_lines = Vec::new();
    for _ in 0..top_pad {
        char_lines.push(Line::from(""));
    }
    for line in &art_lines {
        char_lines.push(Line::from(Span::styled(
            line.as_str(),
            theme::accent_bold(),
        )));
    }

    f.render_widget(
        Paragraph::new(char_lines).alignment(Alignment::Center),
        mascot_cols[1],
    );

    if cols[1].height < 2 {
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
        .unwrap_or("no dev server detected");

    let ws_cfg = crate::workspace::load_workspace_config(&app.workspace);
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Playhouse", theme::accent_bold()),
            Span::styled(" · ", theme::text_dim()),
            Span::styled(task_label, task_style),
            Span::styled(" · ", theme::text_dim()),
            Span::styled(pass, theme::text()),
        ]),
        Line::from(vec![
            Span::styled("  workspace  ", theme::text_muted()),
            Span::styled(
                truncate_path(&app.workspace, cols[1].width as usize / 2),
                theme::text(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  verify URL ", theme::text_muted()),
            Span::styled(server, theme::accent()),
        ]),
        Line::from(vec![
            Span::styled("  /help", theme::accent()),
            Span::styled(" commands · ", theme::text_dim()),
            Span::styled("/", theme::accent()),
            Span::styled(" slash · ", theme::text_dim()),
            Span::styled("@", theme::accent()),
            Span::styled(" files · ", theme::text_dim()),
            Span::styled("Enter", theme::accent()),
            Span::styled(" notes", theme::text_dim()),
        ]),
    ];

    if !ws_cfg.initialized {
        lines.push(Line::from(vec![
            Span::styled("  tip ", theme::text_muted()),
            Span::styled("/init", theme::accent_bold()),
            Span::styled(" to set up .playhouse/", theme::text_muted()),
        ]));
    }

    let avail = cols[1].height as usize;
    for (i, line) in lines.into_iter().take(avail).enumerate() {
        f.render_widget(
            Paragraph::new(line),
            Rect {
                x: cols[1].x,
                y: cols[1].y + i as u16,
                width: cols[1].width,
                height: 1,
            },
        );
    }
}

fn truncate_path(path: &str, max: usize) -> String {
    if path.len() <= max {
        path.to_string()
    } else {
        format!("…{}", &path[path.len().saturating_sub(max.saturating_sub(1))..])
    }
}
