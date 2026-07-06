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
use crate::tui::workspace_status;

/// Single-column header for narrow or short terminals.
pub fn render_compact(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_focused());
    let inner = block.inner(area);
    f.render_widget(block, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }
    let ws = workspace_status::load(&app.workspace);
    let lines = status_lines(app, &ws, inner.width as usize);
    let take = inner.height as usize;
    f.render_widget(Paragraph::new(lines.into_iter().take(take).collect::<Vec<_>>()), inner);
}

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
        .clamp(14, 42) as u16;

    const MASCOT_LEFT_PAD: u16 = 1;
    const MIN_TEXT_COL: u16 = 22;
    let mascot_col_w = (art_width + MASCOT_LEFT_PAD + 2).min(inner.width.saturating_sub(MIN_TEXT_COL));

    // Not enough room for mascot + text: text only.
    if mascot_col_w < art_width.saturating_add(2) || inner.width < MIN_TEXT_COL + 10 {
        let ws = workspace_status::load(&app.workspace);
        let lines = status_lines(app, &ws, inner.width as usize);
        let take = inner.height as usize;
        f.render_widget(Paragraph::new(lines.into_iter().take(take).collect::<Vec<_>>()), inner);
        return;
    }

    let cols = Layout::horizontal([
        Constraint::Length(mascot_col_w),
        Constraint::Min(MIN_TEXT_COL),
    ])
    .split(inner);

    if cols[0].width > 0 && cols[0].height > 0 {
        let mascot_cols = Layout::horizontal([
            Constraint::Length(MASCOT_LEFT_PAD),
            Constraint::Min(1),
        ])
        .split(cols[0]);

        let char_lines: Vec<Line> = art_lines
            .iter()
            .map(|line| Line::from(Span::styled(line.as_str(), theme::mascot_art())))
            .collect();

        f.render_widget(
            Paragraph::new(char_lines).alignment(Alignment::Center),
            mascot_cols[1],
        );
    }

    if cols[1].height < 1 || cols[1].width < 1 {
        return;
    }

    let ws = workspace_status::load(&app.workspace);
    let status = status_lines(app, &ws, cols[1].width as usize);
    let take = cols[1].height as usize;
    f.render_widget(
        Paragraph::new(status.into_iter().take(take).collect::<Vec<_>>()),
        cols[1],
    );
}

fn status_lines(app: &App, ws: &workspace_status::WorkspaceStatus, width: usize) -> Vec<Line<'static>> {
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
    let path_budget = width.saturating_sub(14).max(8);

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
                truncate_path(&app.workspace, path_budget),
                theme::text(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  verify URL ", theme::text_muted()),
            Span::styled(
                truncate_path(server, path_budget),
                theme::accent(),
            ),
        ]),
    ];

    if let (Some(stars), Some(run)) = (ws.last_stars, ws.last_run.clone()) {
        let grade = ws.last_grade.clone().unwrap_or_else(|| "—".into());
        lines.push(Line::from(vec![
            Span::styled("  last score ", theme::text_muted()),
            Span::styled(format!("{stars}★ {grade}"), theme::accent_bold()),
            Span::styled(format!(" · {run}"), theme::text_dim()),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("  last score ", theme::text_muted()),
            Span::styled("none · run ", theme::text_dim()),
            Span::styled("/verify", theme::accent()),
        ]));
    }

    lines.push(Line::from(vec![
        Span::styled("  /help", theme::accent()),
        Span::styled(" · ", theme::text_dim()),
        Span::styled("/", theme::accent()),
        Span::styled(" · ", theme::text_dim()),
        Span::styled("@", theme::accent()),
        Span::styled(" · ", theme::text_dim()),
        Span::styled("Enter", theme::accent()),
        Span::styled(" notes", theme::text_dim()),
    ]));

    if !ws_cfg.initialized {
        lines.push(Line::from(vec![
            Span::styled("  tip ", theme::text_muted()),
            Span::styled("/init", theme::accent_bold()),
            Span::styled(" for .playhouse/", theme::text_muted()),
        ]));
    }

    lines
}

fn truncate_path(path: &str, max: usize) -> String {
    if path.len() <= max {
        path.to_string()
    } else {
        format!("…{}", &path[path.len().saturating_sub(max.saturating_sub(1))..])
    }
}
