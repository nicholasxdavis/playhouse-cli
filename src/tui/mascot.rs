use std::path::{Path, PathBuf};

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::config::playhouse_home;
use crate::tui::theme;

const EMBEDDED: &str = include_str!("../../mascot.txt");

pub fn find_mascot(workspace: &str) -> Option<PathBuf> {
    let candidates = [
        Path::new(workspace).join("mascot.txt"),
        playhouse_home().join("mascot.txt"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

pub fn load_lines(workspace: &str) -> Vec<String> {
    if let Some(path) = find_mascot(workspace) {
        if let Ok(content) = std::fs::read_to_string(path) {
            let lines: Vec<String> = content.lines().map(str::to_string).collect();
            if !lines.is_empty() {
                return lines;
            }
        }
    }
    EMBEDDED.lines().map(str::to_string).collect()
}

pub fn line_count(workspace: &str) -> usize {
    load_lines(workspace).len()
}

/// Header row height for the welcome panel (borders + art).
pub fn welcome_header_height(workspace: &str) -> u16 {
    (line_count(workspace) as u16 + 2).clamp(8, 14)
}

pub fn render_corner(f: &mut Frame, area: Rect, workspace: &str, min_width: u16) {
    let lines = load_lines(workspace);
    if area.width < min_width || area.height < lines.len() as u16 + 4 {
        return;
    }

    let max_line_len = lines
        .iter()
        .map(|l| l.width())
        .max()
        .unwrap_or(0);

    if area.width as usize <= max_line_len + 30 {
        return;
    }

    let x = area.x + area.width.saturating_sub(max_line_len as u16 + 3);
    let y = area.y + 1;
    let art_rect = Rect {
        x,
        y,
        width: max_line_len as u16 + 2,
        height: lines.len() as u16,
    };

    let art_lines: Vec<Line> = lines
        .iter()
        .map(|l| Line::from(Span::styled(l.as_str(), theme::accent_bold())))
        .collect();

    f.render_widget(Paragraph::new(art_lines), art_rect);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_mascot_not_empty() {
        assert!(!EMBEDDED.trim().is_empty());
        assert!(load_lines(".").len() >= 8);
    }
}
