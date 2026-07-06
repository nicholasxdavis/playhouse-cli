use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::{App, FeedRole};
use crate::tui::selection::LineSelection;
use crate::tui::theme;
use crate::tui::ui_blocks;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeedScrollMetrics {
    pub total_lines: usize,
    pub viewport_lines: usize,
    pub max_scroll_top: usize,
}

pub fn measure_feed(app: &App, area: Rect) -> FeedScrollMetrics {
    if area.height == 0 || area.width == 0 {
        return FeedScrollMetrics {
            total_lines: 0,
            viewport_lines: 0,
            max_scroll_top: 0,
        };
    }
    if app.feed.is_empty() {
        return FeedScrollMetrics {
            total_lines: 0,
            viewport_lines: area.height as usize,
            max_scroll_top: 0,
        };
    }
    let (visual, visible_height) = build_visual_lines(app, area);
    let total = visual.len();
    FeedScrollMetrics {
        total_lines: total,
        viewport_lines: visible_height,
        max_scroll_top: total.saturating_sub(visible_height),
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &mut App) {
    theme::clear_area(f, area);
    app.feed_pane_area = area;

    if area.height == 0 || area.width == 0 {
        return;
    }

    if app.feed.is_empty() {
        let hint = Paragraph::new(Line::from(vec![
            Span::styled("  Type ", theme::text_dim()),
            Span::styled("/", theme::accent()),
            Span::styled(" for QA commands · ", theme::text_dim()),
            Span::styled("@", theme::accent()),
            Span::styled(" to attach files · drag to select", theme::text_dim()),
        ]));
        f.render_widget(hint, area);
        app.feed_line_count = 0;
        app.feed_viewport_lines = area.height as usize;
        return;
    }

    let (visual, visible_height) = build_visual_lines(app, area);
    let total = visual.len();
    app.feed_line_count = total;
    app.feed_viewport_lines = visible_height;

    let max_top = total.saturating_sub(visible_height);
    if app.feed_stick_bottom {
        app.feed_scroll_pos = max_top as f32;
        app.feed_scroll_target = max_top as f32;
    }
    app.clamp_feed_scroll(max_top);

    let scroll_y = app.feed_scroll_line().min(u16::MAX as usize) as u16;

    const SCROLLBAR_WIDTH: u16 = 1;
    let use_scrollbar = max_top > 0;
    let content_width = if use_scrollbar {
        area.width.saturating_sub(SCROLLBAR_WIDTH)
    } else {
        area.width
    };

    let content_area = if use_scrollbar {
        Layout::horizontal([
            Constraint::Length(content_width),
            Constraint::Length(SCROLLBAR_WIDTH),
        ])
        .split(area)[0]
    } else {
        area
    };

    let paragraph = Paragraph::new(apply_selection_to_lines(
        visual,
        scroll_y as usize,
        visible_height,
        app.feed_text_selection,
    ))
    .scroll((scroll_y, 0));
    f.render_widget(paragraph, content_area);

    if use_scrollbar {
        let scrollbar_col = Rect {
            x: area.x + content_width,
            y: area.y,
            width: SCROLLBAR_WIDTH,
            height: area.height,
        };
        app.feed_scrollbar_area = scrollbar_col;
        if let Some(layout) =
            scrollbar_layout(scrollbar_col, total, visible_height, app.feed_scroll_pos)
        {
            app.feed_scrollbar_track_area = layout.track_area;
            render_scrollbar(f, scrollbar_col, &layout);
        }
    } else {
        app.feed_scrollbar_area = Rect::default();
        app.feed_scrollbar_track_area = Rect::default();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollbarLayout {
    pub track_area: Rect,
    pub thumb_top: f32,
    pub thumb_height: f32,
    pub max_scroll: f32,
}

pub fn scrollbar_layout(
    area: Rect,
    total: usize,
    viewport: usize,
    pos: f32,
) -> Option<ScrollbarLayout> {
    if area.height == 0 || total <= viewport {
        return None;
    }
    let max_scroll = (total - viewport) as f32;
    let h = area.height as usize;
    let (track_area, track_rows) = if h >= 3 {
        (
            Rect {
                x: area.x,
                y: area.y + 1,
                width: area.width,
                height: area.height.saturating_sub(2),
            },
            area.height.saturating_sub(2) as f32,
        )
    } else {
        (area, area.height as f32)
    };
    if track_rows < 1.0 {
        return None;
    }
    let thumb_frac = viewport as f32 / total as f32;
    let thumb_height = (track_rows * thumb_frac).clamp(1.0, track_rows);
    let travel = (track_rows - thumb_height).max(0.0);
    let thumb_top = if max_scroll > 0.0 {
        (pos / max_scroll) * travel
    } else {
        0.0
    };
    Some(ScrollbarLayout {
        track_area,
        thumb_top,
        thumb_height,
        max_scroll,
    })
}

fn render_scrollbar(f: &mut Frame, area: Rect, layout: &ScrollbarLayout) {
    let h = area.height as usize;
    if h == 0 {
        return;
    }
    if h >= 3 {
        f.render_widget(
            Paragraph::new("▲").style(theme::text_dim()),
            Rect::new(area.x, area.y, 1, 1),
        );
        f.render_widget(
            Paragraph::new("▼").style(theme::text_dim()),
            Rect::new(area.x, area.y + area.height - 1, 1, 1),
        );
    }
    for i in 0..layout.track_area.height {
        let rel = i as f32;
        let on_thumb =
            rel >= layout.thumb_top - 0.001 && rel < layout.thumb_top + layout.thumb_height;
        let sym = if on_thumb { "█" } else { "│" };
        let style = if on_thumb {
            theme::accent()
        } else {
            theme::text_dim()
        };
        f.render_widget(
            Paragraph::new(sym).style(style),
            Rect::new(
                layout.track_area.x,
                layout.track_area.y + i,
                1,
                1,
            ),
        );
    }
}

fn build_visual_lines(app: &App, area: Rect) -> (Vec<Line<'static>>, usize) {
    const SCROLLBAR_WIDTH: u16 = 1;
    let content_width = area.width.saturating_sub(SCROLLBAR_WIDTH);
    let max_line_width = content_width.saturating_sub(1).max(1) as usize;
    let visible_height = area.height as usize;

    let mut visual = Vec::new();
    for entry in &app.feed {
        if !visual.is_empty() {
            visual.push(Line::from(""));
        }
        let label = match entry.role {
            FeedRole::User => "You",
            FeedRole::System => "Playhouse",
        };
        visual.push(Line::from(vec![Span::styled(
            format!("  {label}  "),
            theme::accent_bold(),
        )]));
        visual.extend(ui_blocks::render_blocks(&entry.blocks, max_line_width, app.tick_count));
    }
    (visual, visible_height)
}

pub fn build_feed_plain_lines(app: &App, area: Rect) -> Vec<String> {
    let (lines, _) = build_visual_lines(app, area);
    lines
        .into_iter()
        .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect::<String>())
        .collect()
}

pub fn feed_pos_to_line_col(
    area: Rect,
    scroll_y: usize,
    row: u16,
    col: u16,
) -> Option<(usize, usize)> {
    if !area.contains(ratatui::layout::Position::new(col, row)) {
        return None;
    }
    let rel_row = row.saturating_sub(area.y) as usize + scroll_y;
    let rel_col = col.saturating_sub(area.x) as usize;
    Some((rel_row, rel_col))
}

fn apply_selection_to_lines(
    lines: Vec<Line<'static>>,
    scroll_top: usize,
    viewport: usize,
    selection: Option<LineSelection>,
) -> Vec<Line<'static>> {
    let Some(sel) = selection else {
        return lines;
    };
    if sel.is_empty() {
        return lines;
    }
    lines
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            if i < scroll_top || i >= scroll_top + viewport {
                return line;
            }
            let vis_line = i;
            let plain: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            let chars: Vec<char> = plain.chars().collect();
            let mut spans = Vec::new();
            for (col, ch) in chars.iter().enumerate() {
                let in_sel = sel.contains(vis_line, col);
                let base = line
                    .spans
                    .first()
                    .map(|s| s.style)
                    .unwrap_or_else(theme::text);
                let style = if in_sel {
                    Style::default()
                        .fg(theme::get_bg_color())
                        .bg(theme::get_text_primary())
                        .add_modifier(Modifier::BOLD)
                } else {
                    base
                };
                spans.push(Span::styled(ch.to_string(), style));
            }
            if spans.is_empty() {
                line
            } else {
                Line::from(spans)
            }
        })
        .collect()
}
