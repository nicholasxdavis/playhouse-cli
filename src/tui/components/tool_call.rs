use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::tui::spinner;
use crate::tui::theme;
use crate::tui::text_box;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolStatus {
    Running,
    Success,
    Error,
}

/// Context for rendering a tool call row in the TUI feed.
pub struct ToolCallRenderCtx<'a> {
    pub name: &'a str,
    pub status: &'a ToolStatus,
    pub summary: &'a str,
    pub detail: Option<&'a str>,
    pub max_width: usize,
    pub tick: u64,
}

pub fn render(ctx: &ToolCallRenderCtx<'_>) -> Vec<Line<'static>> {
    let (icon, status_style) = match ctx.status {
        ToolStatus::Running => {
            let spin = spinner::frame(ctx.tick);
            (spin, theme::status_busy())
        }
        ToolStatus::Success => ("+", theme::status_ready()),
        ToolStatus::Error => ("x", theme::status_error()),
    };

    let dots = ".".repeat(ctx.tick as usize / 10 % 4);
    let summary_display = if matches!(ctx.status, ToolStatus::Running) {
        format!("{}{dots}", ctx.summary)
    } else {
        ctx.summary.to_string()
    };

    let mut out = vec![Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(format!("{icon} "), status_style),
        Span::styled(ctx.name.to_string(), theme::accent_bold()),
        Span::styled(" | ", theme::text_dim()),
        Span::styled(summary_display, theme::text()),
    ])];

    if let Some(d) = ctx.detail {
        for wl in text_box::wrap_text(d, ctx.max_width.saturating_sub(4)) {
            out.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(wl, theme::system_detail_text()),
            ]));
        }
    }

    out
}
