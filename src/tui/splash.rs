use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use crate::config::playhouse_home;
use crate::tui::ascii_asset;
use crate::tui::spinner;
use crate::tui::theme;

const EMBEDDED: &str = include_str!("../../splash.txt");
const LOGO_LINES: usize = 8;

const STEPS_NORMAL: &[&str] = &[
    "Loading workspace",
    "Reading preferences",
    "Preparing interface",
    "Almost ready",
];

const STEPS_FIRST: &[&str] = &[
    "Welcome to Playhouse",
    "Preparing your QA workspace",
    "Checking bundled tools",
    "Loading interface",
    "Ready when you are",
];

pub fn skip_requested() -> bool {
    matches!(
        std::env::var("PLAYHOUSE_NO_SPLASH").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

pub fn is_first_run(workspace: &str) -> bool {
    let cfg = crate::workspace::load_workspace_config(workspace);
    if !cfg.initialized {
        return true;
    }
    !playhouse_home().join("tui_launched").exists()
}

pub fn mark_launched() {
    let dir = playhouse_home();
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(
        dir.join("tui_launched"),
        env!("CARGO_PKG_VERSION"),
    );
}

pub fn load_lines(workspace: &str) -> Vec<String> {
    let lines = ascii_asset::load_asset_lines(workspace, "splash.txt", EMBEDDED);
    trim_leading_margin(&lines)
}

/// Strip shared leading whitespace; keep only the main logo rows (drop orphan tail).
fn trim_leading_margin(lines: &[String]) -> Vec<String> {
    let indent = lines
        .iter()
        .filter(|l| l.chars().any(|c| !c.is_whitespace()))
        .map(|l| l.chars().take_while(|c| *c == ' ').count())
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .map(|l| {
            if l.len() <= indent {
                String::new()
            } else {
                l[indent..].to_string()
            }
        })
        .filter(|l| l.chars().any(|c| !c.is_whitespace()))
        .take(LOGO_LINES)
        .collect()
}

pub fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    _workspace: &str,
    first_run: bool,
) -> io::Result<()> {
    let lines = load_lines(_workspace);
    let total_ticks = if first_run { 50u64 } else { 36u64 };
    let steps = if first_run { STEPS_FIRST } else { STEPS_NORMAL };
    let tick_rate = Duration::from_millis(40);
    let mut tick: u64 = 0;

    while tick < total_ticks {
        terminal.autoresize()?;
        terminal.draw(|f| draw_frame(f, &lines, tick, total_ticks, steps, first_run))?;

        if !event::poll(tick_rate)? {
            tick += 1;
            continue;
        }

        let mut skip = false;
        let mut resized = false;
        let mut events = 0u32;
        loop {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    skip = true;
                    break;
                }
                Event::Resize(_, _) => resized = true,
                _ => {}
            }
            events += 1;
            if events > 64 || !event::poll(Duration::from_millis(0))? {
                break;
            }
        }

        if skip {
            break;
        }
        if resized {
            continue;
        }
        tick += 1;
    }

    terminal.clear()?;
    Ok(())
}

fn draw_frame(
    f: &mut Frame,
    lines: &[String],
    tick: u64,
    total_ticks: u64,
    steps: &[&str],
    first_run: bool,
) {
    let area = f.area();
    if area.width == 0 || area.height == 0 {
        return;
    }

    f.render_widget(Clear, area);
    f.render_widget(
        Block::default().style(theme::text().bg(theme::get_bg_color())),
        area,
    );

    if area.width < 24 || area.height < 10 {
        draw_compact(f, area, tick, total_ticks, steps);
        return;
    }

    let logo_w = lines
        .iter()
        .map(|l| l.width())
        .max()
        .unwrap_or(40)
        .clamp(20, 88) as u16;

    let logo_h = lines.len() as u16;
    let chrome = 4u16 + if first_run { 1 } else { 0 };
    let ideal_h = logo_h + chrome + 2;
    let panel_h = ideal_h.min(area.height.saturating_sub(2)).max(chrome + 3);
    let panel_w = logo_w.saturating_add(4).min(area.width.saturating_sub(2)).max(28);

    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(panel_h),
        Constraint::Fill(1),
    ])
    .split(area);

    let horizontal = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(panel_w),
        Constraint::Fill(1),
    ])
    .split(vertical[1]);

    let panel = horizontal[1];
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border_focused())
            .style(theme::text().bg(theme::get_bg_surface())),
        panel,
    );

    let inner = Block::default().borders(Borders::NONE).inner(panel);
    let mut rows = Vec::new();
    if first_run {
        rows.push(Constraint::Length(1));
    }
    let art_h = inner
        .height
        .saturating_sub(chrome)
        .max(1)
        .min(logo_h);
    rows.push(Constraint::Length(art_h));
    rows.extend([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)]);
    let sections = Layout::vertical(rows).split(inner);

    let art_idx = if first_run { 1 } else { 0 };
    let status_idx = art_idx + 1;

    if first_run {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "first launch",
                theme::accent_bold().add_modifier(Modifier::ITALIC),
            )))
            .alignment(Alignment::Center),
            sections[0],
        );
    }

    let revealed = ((tick * (lines.len() as u64 + 1)) / (total_ticks * 2 / 3))
        .min(lines.len() as u64) as usize;
    let partial = tick % 4;
    let art_lines: Vec<Line> = lines
        .iter()
        .take(art_h as usize)
        .enumerate()
        .map(|(i, line)| {
            if i < revealed {
                Line::from(Span::styled(line.as_str(), theme::splash_art()))
            } else if i == revealed && partial > 0 {
                let cut = (line.len() * partial as usize / 4).min(line.len());
                let (bright, dim) = line.split_at(cut);
                Line::from(vec![
                    Span::styled(bright, theme::splash_art()),
                    Span::styled(dim, theme::splash_art_dim()),
                ])
            } else {
                Line::from(Span::styled(line.as_str(), theme::splash_art_dim()))
            }
        })
        .collect();

    f.render_widget(
        Paragraph::new(art_lines).alignment(Alignment::Center),
        sections[art_idx],
    );

    let step_idx = ((tick * steps.len() as u64) / total_ticks.max(1))
        .min(steps.len().saturating_sub(1) as u64) as usize;
    let spin = spinner::frame(tick);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("{spin} "), theme::accent_bold()),
            Span::styled(steps[step_idx], theme::text()),
            Span::styled("…", theme::text_dim()),
        ]))
        .alignment(Alignment::Center),
        sections[status_idx],
    );

    let percent = ((tick * 100) / total_ticks.max(1)).min(100) as u16;
    f.render_widget(
        Gauge::default()
            .gauge_style(theme::accent())
            .percent(percent)
            .label(Span::styled(format!("{percent}%"), theme::text_muted())),
        sections[status_idx + 1],
    );

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "QA CLI for humans",
            theme::text_muted(),
        )))
        .alignment(Alignment::Center),
        sections[status_idx + 2],
    );

    let version = env!("CARGO_PKG_VERSION");
    let hint = if tick > 4 { " · any key to skip" } else { "" };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("v{version}"), theme::text_dim()),
            Span::styled(hint, theme::text_dim()),
        ]))
        .alignment(Alignment::Center),
        sections[status_idx + 3],
    );
}

fn draw_compact(f: &mut Frame, area: Rect, tick: u64, total_ticks: u64, steps: &[&str]) {
    let step_idx = ((tick * steps.len() as u64) / total_ticks.max(1))
        .min(steps.len().saturating_sub(1) as u64) as usize;
    let spin = spinner::frame(tick);
    let percent = ((tick * 100) / total_ticks.max(1)).min(100) as u16;

    let inner = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(area);

    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("Playhouse", theme::splash_art())),
            Line::from(vec![
                Span::styled(format!("{spin} "), theme::accent_bold()),
                Span::styled(steps[step_idx], theme::text()),
            ]),
        ])
        .alignment(Alignment::Center),
        inner[1],
    );

    f.render_widget(
        Gauge::default()
            .gauge_style(theme::accent())
            .percent(percent),
        inner[2],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_splash_not_empty() {
        assert!(!EMBEDDED.trim().is_empty());
        assert!(load_lines(".").len() >= 6);
    }

    #[test]
    fn trim_margin_removes_shared_indent() {
        let lines = vec!["    aa".into(), "    bb".into()];
        let trimmed = trim_leading_margin(&lines);
        assert_eq!(trimmed.len(), 2);
        assert_eq!(trimmed[0], "aa");
    }

    #[test]
    fn logo_capped_at_eight_lines() {
        let lines: Vec<String> = (0..12).map(|i| format!("    line{i}")).collect();
        assert_eq!(trim_leading_margin(&lines).len(), LOGO_LINES);
    }
}
