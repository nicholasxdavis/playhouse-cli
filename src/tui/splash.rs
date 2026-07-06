use std::io::{self, Stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use crate::config::playhouse_home;
use crate::tui::spinner;
use crate::tui::theme;

const EMBEDDED: &str = include_str!("../../splash.txt");

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

fn find_splash(workspace: &str) -> Option<PathBuf> {
    let candidates = [
        Path::new(workspace).join("splash.txt"),
        playhouse_home().join("splash.txt"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

pub fn load_lines(workspace: &str) -> Vec<String> {
    if let Some(path) = find_splash(workspace) {
        if let Ok(content) = std::fs::read_to_string(path) {
            let lines: Vec<String> = content.lines().map(str::to_string).collect();
            if !lines.is_empty() {
                return lines;
            }
        }
    }
    EMBEDDED.lines().map(str::to_string).collect()
}

pub fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    workspace: &str,
    first_run: bool,
) -> io::Result<()> {
    let lines = load_lines(workspace);
    let total_ticks = if first_run { 58u64 } else { 42u64 };
    let steps = if first_run { STEPS_FIRST } else { STEPS_NORMAL };
    let tick_rate = Duration::from_millis(48);
    let mut tick: u64 = 0;

    while tick < total_ticks {
        terminal.draw(|f| draw_frame(f, &lines, tick, total_ticks, steps, first_run))?;

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => break,
                Event::Resize(_, _) => {}
                _ => {}
            }
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
    f.render_widget(
        Block::default().style(theme::text().bg(theme::get_bg_color())),
        area,
    );

    if area.width < 24 || area.height < 12 {
        draw_compact(f, area, tick, total_ticks, steps);
        return;
    }

    let logo_width = lines
        .iter()
        .map(|l| l.width())
        .max()
        .unwrap_or(40) as u16;

    let logo_height = lines.len() as u16;
    let chrome = 8u16 + if first_run { 1 } else { 0 };
    let block_height = logo_height + chrome;

    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(block_height),
        Constraint::Fill(1),
    ])
    .split(area);

    let horizontal = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(logo_width.min(area.width.saturating_sub(4)).max(24)),
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
    let sections = Layout::vertical([
        Constraint::Length(if first_run { 1 } else { 0 }),
        Constraint::Min(logo_height),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);

    let mut row = 0usize;
    if first_run {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  first launch",
                theme::accent_bold().add_modifier(Modifier::ITALIC),
            )))
            .alignment(Alignment::Center),
            sections[row],
        );
        row += 1;
    }

    let revealed_lines = ((tick * (lines.len() as u64 + 1)) / (total_ticks * 2 / 3))
        .min(lines.len() as u64) as usize;
    let partial = tick % 4;
    let art_lines: Vec<Line> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if i < revealed_lines {
                Line::from(Span::styled(line.as_str(), theme::accent_bold()))
            } else if i == revealed_lines && partial > 0 {
                let cut = (line.len() * partial as usize / 4).min(line.len());
                let (bright, dim) = line.split_at(cut);
                Line::from(vec![
                    Span::styled(bright, theme::accent_bold()),
                    Span::styled(dim, theme::text_dim()),
                ])
            } else {
                Line::from(Span::styled(line.as_str(), theme::text_dim()))
            }
        })
        .collect();

    f.render_widget(
        Paragraph::new(art_lines).alignment(Alignment::Left),
        sections[row],
    );
    row += 1;

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
        sections[row],
    );
    row += 1;

    let percent = ((tick * 100) / total_ticks.max(1)).min(100) as u16;
    f.render_widget(
        Gauge::default()
            .gauge_style(theme::accent())
            .percent(percent)
            .label(Span::styled(
                format!("{percent}%"),
                theme::text_muted(),
            )),
        sections[row],
    );
    row += 1;

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "QA CLI for humans",
            theme::text_muted(),
        )))
        .alignment(Alignment::Center),
        sections[row],
    );
    row += 1;

    let version = env!("CARGO_PKG_VERSION");
    let hint = if tick > 6 {
        " · any key to skip"
    } else {
        ""
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("v{version}"), theme::text_dim()),
            Span::styled(hint, theme::text_dim()),
        ]))
        .alignment(Alignment::Center),
        sections[row],
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
            Line::from(Span::styled("Playhouse", theme::accent_bold())),
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
        assert!(load_lines(".").len() >= 8);
    }
}
