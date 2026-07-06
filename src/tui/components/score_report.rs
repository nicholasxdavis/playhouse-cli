use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::engine_status::{self, EngineRunKind};
use crate::score::{EngineResult, PlayhouseScore};
use crate::tui::theme;
use crate::tui::text_box;

pub fn render(
    score: &PlayhouseScore,
    exit_code: i32,
    engines: &[EngineResult],
    max_width: usize,
    reveal: u64,
) -> Vec<Line<'static>> {
    let mut out = render_header(score);
    out.push(Line::from(""));
    out.extend(render_category_table(score, max_width));
    out.push(Line::from(""));
    out.extend(render_engine_list(engines, reveal));
    out.extend(render_why_section(score, reveal));
    out.push(Line::from(""));
    out.extend(render_footer(exit_code, reveal));
    out
}

fn render_header(score: &PlayhouseScore) -> Vec<Line<'static>> {
    let overall_style = stars_style(score.stars);
    vec![Line::from(vec![
        Span::styled("  Playhouse Stars  ", theme::accent_bold()),
        Span::styled(
            format!("{} / 100", score.stars),
            overall_style.add_modifier(ratatui::style::Modifier::BOLD),
        ),
        Span::styled(format!("  {}  ", score.grade_emoji), theme::text()),
        Span::styled(score.grade.clone(), overall_style),
    ])]
}

fn render_category_table(score: &PlayhouseScore, max_width: usize) -> Vec<Line<'static>> {
    let col1 = 22usize;
    let col2 = 8usize;
    let mut out = vec![
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(pad_right("Category", col1), theme::accent()),
            Span::styled(pad_right("Score", col2), theme::accent()),
            Span::styled("Summary", theme::accent()),
        ]),
        Line::from(Span::styled(
            format!("  {}", "-".repeat(max_width.saturating_sub(4).min(60))),
            theme::border(),
        )),
    ];
    for cat in &score.categories {
        if cat.skipped {
            out.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(pad_right(&cat.label, col1), theme::todo_item_skipped()),
                Span::styled(pad_right("skip", col2), theme::todo_item_skipped()),
                Span::styled(cat.summary.clone(), theme::todo_item_skipped()),
            ]));
            continue;
        }
        let row_style = stars_style(cat.stars);
        out.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(pad_right(&cat.label, col1), theme::text()),
            Span::styled(pad_right(&format!("{}/100", cat.stars), col2), row_style),
            Span::styled(cat.summary.clone(), theme::text_muted()),
        ]));
    }
    out
}

fn render_engine_list(engines: &[EngineResult], reveal: u64) -> Vec<Line<'static>> {
    if reveal <= 4 {
        return Vec::new();
    }
    let mut out = vec![Line::from(Span::styled("  Engines", theme::accent_bold()))];
    for er in engines {
        let (icon, style) = engine_row_style(er);
        let label = engine_status::engine_label(er);
        out.push(Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled(format!("{icon} "), style),
            Span::styled(label, theme::text_muted()),
        ]));
    }
    out
}

fn engine_row_style(er: &EngineResult) -> (&'static str, Style) {
    match EngineRunKind::from_engine(er) {
        EngineRunKind::Ran if er.exit_code == 0 => ("+", theme::status_ready()),
        EngineRunKind::Ran => ("x", theme::status_error()),
        EngineRunKind::ExplicitSkip => ("-", theme::todo_item_skipped()),
        EngineRunKind::ImplicitPenalty => ("!", theme::status_error()),
    }
}

fn render_why_section(score: &PlayhouseScore, reveal: u64) -> Vec<Line<'static>> {
    if reveal <= 8 || score.why.is_empty() {
        return Vec::new();
    }
    let mut out = vec![Line::from(""), Line::from(Span::styled("  Why", theme::accent_bold()))];
    let ticks_per_line = 10u64;
    for (i, line) in score.why.iter().enumerate() {
        let line_start = 8 + i as u64 * ticks_per_line;
        if reveal < line_start {
            break;
        }
        let chars_visible = ((reveal - line_start) * 3).min(line.len() as u64) as usize;
        let text: String = line.chars().take(chars_visible).collect();
        let cursor = if chars_visible < line.len() { "|" } else { "" };
        out.push(Line::from(vec![
            Span::styled("    - ", theme::accent()),
            Span::styled(format!("{text}{cursor}"), theme::text()),
        ]));
    }
    out
}

fn render_footer(exit_code: i32, reveal: u64) -> Vec<Line<'static>> {
    let result_style = if exit_code == 0 {
        theme::status_ready()
    } else {
        theme::status_error()
    };
    let mut out = vec![Line::from(vec![
        Span::styled("  Report  ", theme::text_dim()),
        Span::styled(".playhouse/reports/score.json", theme::code_style()),
    ])];
    if reveal > 2 {
        let label = if exit_code == 0 {
            "  Verify passed"
        } else {
            "  Verify failed"
        };
        out.push(Line::from(vec![
            Span::styled(label, result_style),
            Span::styled(format!(" | exit {exit_code}"), theme::text_muted()),
        ]));
    }
    out
}

fn stars_style(stars: u8) -> Style {
    if stars >= 75 {
        theme::status_ready()
    } else if stars >= 60 {
        theme::status_accent()
    } else {
        theme::status_error()
    }
}

fn pad_right(s: &str, width: usize) -> String {
    let w = text_box::width(s);
    if w >= width {
        format!("{} ", text_box::truncate(s, width))
    } else {
        format!("{}{} ", s, " ".repeat(width - w))
    }
}
