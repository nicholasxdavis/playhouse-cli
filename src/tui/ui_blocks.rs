use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::score::{EngineResult, PlayhouseScore};
use crate::tui::components::{score_report, tool_call};
use crate::tui::spinner;
use crate::tui::theme;
use crate::tui::text_box;

pub use tool_call::ToolStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TodoStatus {
    Pending,
    Active,
    Done,
    Warn,
    Skipped,
}

#[derive(Clone, Debug)]
pub struct TodoItem {
    pub text: String,
    pub status: TodoStatus,
    pub detail: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ContentBlock {
    Text { content: String },
    Code { content: String },
    TodoList { title: String, items: Vec<TodoItem> },
    ToolCall {
        name: String,
        status: ToolStatus,
        summary: String,
        detail: Option<String>,
    },
    ScoreReport {
        score: PlayhouseScore,
        exit_code: i32,
        engines: Vec<EngineResult>,
        reveal_tick: u64,
    },
}

impl ContentBlock {
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text {
            content: content.into(),
        }
    }

    pub fn code(content: impl Into<String>) -> Self {
        Self::Code {
            content: content.into(),
        }
    }

    pub fn tool_running(name: impl Into<String>, summary: impl Into<String>) -> Self {
        Self::ToolCall {
            name: name.into(),
            status: ToolStatus::Running,
            summary: summary.into(),
            detail: None,
        }
    }

    pub fn tool_done(
        name: impl Into<String>,
        summary: impl Into<String>,
        success: bool,
        detail: Option<String>,
    ) -> Self {
        Self::ToolCall {
            name: name.into(),
            status: if success {
                ToolStatus::Success
            } else {
                ToolStatus::Error
            },
            summary: summary.into(),
            detail,
        }
    }

    pub fn todo_list(title: impl Into<String>, items: Vec<TodoItem>) -> Self {
        Self::TodoList {
            title: title.into(),
            items,
        }
    }

    pub fn score_report(score: PlayhouseScore, exit_code: i32, engines: Vec<EngineResult>) -> Self {
        Self::ScoreReport {
            score,
            exit_code,
            engines,
            reveal_tick: 0,
        }
    }
}

pub fn render_blocks(blocks: &[ContentBlock], max_width: usize, tick: u64) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            lines.push(Line::from(""));
        }
        lines.extend(render_block(block, max_width, tick));
    }
    lines
}

pub fn render_block(block: &ContentBlock, max_width: usize, tick: u64) -> Vec<Line<'static>> {
    match block {
        ContentBlock::Text { content } => render_narrative(content, max_width),
        ContentBlock::Code { content } => render_code_preview(content, max_width),
        ContentBlock::TodoList { title, items } => render_todo_list(title, items, max_width, tick),
        ContentBlock::ToolCall {
            name,
            status,
            summary,
            detail,
        } => tool_call::render(&tool_call::ToolCallRenderCtx {
            name,
            status,
            summary,
            detail: detail.as_deref(),
            max_width,
            tick,
        }),
        ContentBlock::ScoreReport {
            score,
            exit_code,
            engines,
            reveal_tick,
        } => score_report::render(
            score,
            *exit_code,
            engines,
            max_width,
            tick.saturating_sub(*reveal_tick),
        ),
    }
}

fn render_narrative(content: &str, max_width: usize) -> Vec<Line<'static>> {
    let width = max_width.max(20);
    let mut out = Vec::new();
    for paragraph in content.split('\n') {
        if paragraph.trim().is_empty() {
            out.push(Line::from(""));
            continue;
        }
        for wl in text_box::wrap_text(paragraph, width) {
            out.push(Line::from(Span::styled(wl, theme::text())));
        }
    }
    out
}

fn render_code_preview(content: &str, max_width: usize) -> Vec<Line<'static>> {
    let width = max_width.saturating_sub(4).max(20);
    let style = theme::code_style();
    let mut out = Vec::new();
    for line in content.lines().take(64) {
        for wl in text_box::wrap_text(line, width) {
            out.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(wl, style),
            ]));
        }
    }
    out
}

fn render_todo_list(title: &str, items: &[TodoItem], max_width: usize, tick: u64) -> Vec<Line<'static>> {
    if items.is_empty() {
        return Vec::new();
    }
    let done = items
        .iter()
        .filter(|i| {
            matches!(
                i.status,
                TodoStatus::Done | TodoStatus::Skipped | TodoStatus::Warn
            )
        })
        .count();
    let total = items.len();
    let pct = done as f64 / total as f64;
    let bar_w = 14usize;
    let filled = (pct * bar_w as f64).round() as usize;
    let bar = format!(
        "{}{}",
        "#".repeat(filled),
        "-".repeat(bar_w.saturating_sub(filled))
    );

    let mut out = Vec::new();
    out.push(Line::from(vec![
        Span::styled(format!("  {title}"), theme::todo_panel_label()),
        Span::styled("  ", Style::default()),
        Span::styled(bar, theme::progress_bar(filled, bar_w)),
        Span::styled(format!("  {done}/{total}"), theme::text_muted()),
    ]));

    for item in items {
        let (mark, style) = match item.status {
            TodoStatus::Pending => ("o", theme::todo_item_pending()),
            TodoStatus::Active => (spinner::frame(tick), theme::todo_item_active()),
            TodoStatus::Done => ("+", theme::todo_item_done()),
            TodoStatus::Warn => ("~", theme::status_warn()),
            TodoStatus::Skipped => ("-", theme::todo_item_skipped()),
        };
        let body = text_box::truncate(&item.text, max_width.saturating_sub(10));
        out.push(Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled(format!("{mark} "), style),
            Span::styled(body, style),
        ]));
        if let Some(detail) = &item.detail {
            let detail_style = todo_detail_style(&item.status, detail);
            let detail_text = if item.status == TodoStatus::Active {
                let dots = ".".repeat(tick as usize / 8 % 4);
                format!("{detail}{dots}")
            } else {
                detail.clone()
            };
            out.push(Line::from(vec![
                Span::styled("       ", Style::default()),
                Span::styled(
                    text_box::truncate(&detail_text, max_width.saturating_sub(10)),
                    detail_style,
                ),
            ]));
        }
    }
    out
}

fn todo_detail_style(status: &TodoStatus, detail: &str) -> Style {
    match status {
        TodoStatus::Active => theme::todo_item_active(),
        TodoStatus::Skipped => theme::todo_item_skipped(),
        TodoStatus::Warn => theme::status_warn(),
        TodoStatus::Done if detail.contains("failed") || detail.contains("vulns") => {
            theme::status_error()
        }
        TodoStatus::Done => theme::system_detail_text(),
        TodoStatus::Pending => theme::text_dim(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::{CategoryScore, PlayhouseScore};

    #[test]
    fn render_text_block_wraps_lines() {
        let block = ContentBlock::text("hello world");
        let lines = render_block(&block, 40, 0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn render_todo_list_shows_progress() {
        let items = vec![
            TodoItem {
                text: "Step one".into(),
                status: TodoStatus::Done,
                detail: None,
            },
            TodoItem {
                text: "Step two".into(),
                status: TodoStatus::Pending,
                detail: None,
            },
        ];
        let block = ContentBlock::todo_list("Verify", items);
        let lines = render_block(&block, 60, 0);
        let text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.clone()))
            .collect();
        assert!(text.contains("Verify"));
        assert!(text.contains("1/2"));
    }

    #[test]
    fn render_score_report_includes_stars() {
        let score = PlayhouseScore {
            stars: 100,
            grade: "Production Ready".into(),
            grade_emoji: "*****".into(),
            passed: true,
            categories: vec![CategoryScore {
                id: "tools".into(),
                label: "Toolchain".into(),
                stars: 100,
                weight: 0.10,
                summary: "4/4 tools ready".into(),
                details: vec![],
                skipped: false,
            }],
            why: vec![],
            methodology: String::new(),
        };
        let block = ContentBlock::score_report(score, 0, vec![]);
        let lines = render_block(&block, 60, 20);
        let text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.clone()))
            .collect();
        assert!(text.contains("100"));
        assert!(text.contains("Production Ready"));
    }
}
