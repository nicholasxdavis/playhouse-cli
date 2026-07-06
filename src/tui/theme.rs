#![allow(dead_code)]

use ratatui::style::{Color, Modifier, Style};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

pub struct ThemeState {
    pub is_light_theme: bool,
    pub accent_rgb: (u8, u8, u8),
}

static THEME_STATE: RwLock<ThemeState> = RwLock::new(ThemeState {
    is_light_theme: false,
    accent_rgb: (42, 157, 143),
});

fn config_path() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".playhouse_theme")
}

pub fn load_config() {
    if let Ok(content) = fs::read_to_string(config_path()) {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("theme_mode=") {
                set_light_theme(val.trim().eq_ignore_ascii_case("light"));
            }
        }
    }
}

pub fn set_light_theme(light: bool) {
    if let Ok(mut state) = THEME_STATE.write() {
        state.is_light_theme = light;
    }
}

pub fn set_accent_from_string(input: &str) -> Result<String, String> {
    let s = input.trim().to_lowercase();
    let (r, g, b, name) = match s.as_str() {
        "teal" | "default" | "reset" => (42, 157, 143, "Teal"),
        "red" => (231, 76, 60, "Red"),
        "orange" | "amber" => (100, 175, 195, "Cyan"),
        "green" => (46, 204, 113, "Green"),
        "blue" => (52, 152, 219, "Blue"),
        "purple" => (155, 89, 182, "Purple"),
        "cyan" => (0, 255, 255, "Cyan"),
        _ => {
            let hex = s.strip_prefix('#').unwrap_or(&s);
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "bad hex")?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "bad hex")?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "bad hex")?;
                (r, g, b, "Custom")
            } else {
                return Err(format!("Unknown color: {input}"));
            }
        }
    };
    if let Ok(mut state) = THEME_STATE.write() {
        state.accent_rgb = (r, g, b);
    }
    Ok(name.to_string())
}

pub fn is_light_theme() -> bool {
    THEME_STATE
        .read()
        .map(|s| s.is_light_theme)
        .unwrap_or(false)
}

pub fn get_accent_color() -> Color {
    if let Ok(state) = THEME_STATE.read() {
        Color::Rgb(state.accent_rgb.0, state.accent_rgb.1, state.accent_rgb.2)
    } else {
        Color::Rgb(42, 157, 143)
    }
}

pub fn get_bg_color() -> Color {
    if is_light_theme() {
        Color::Rgb(248, 248, 252)
    } else {
        Color::Rgb(18, 18, 24)
    }
}

pub fn get_bg_surface() -> Color {
    if is_light_theme() {
        Color::Rgb(235, 235, 242)
    } else {
        Color::Rgb(26, 26, 36)
    }
}

pub fn get_bg_highlight() -> Color {
    if is_light_theme() {
        Color::Rgb(220, 220, 230)
    } else {
        Color::Rgb(42, 42, 56)
    }
}

pub fn get_text_primary() -> Color {
    if is_light_theme() {
        Color::Rgb(20, 20, 30)
    } else {
        Color::Rgb(245, 245, 250)
    }
}

pub fn get_text_secondary() -> Color {
    if is_light_theme() {
        Color::Rgb(80, 80, 100)
    } else {
        Color::Rgb(190, 190, 210)
    }
}

pub fn get_text_muted() -> Color {
    if is_light_theme() {
        Color::Rgb(130, 130, 150)
    } else {
        Color::Rgb(145, 145, 165)
    }
}

pub const SUCCESS: Color = Color::Rgb(72, 199, 176);
pub const ERROR: Color = Color::Rgb(210, 105, 120);
pub const ACCENT_MUTED: Color = Color::Rgb(100, 175, 195);

pub fn status_busy() -> Style {
    Style::default().fg(Color::Rgb(140, 190, 210))
}

pub fn status_accent() -> Style {
    Style::default().fg(ACCENT_MUTED)
}

pub fn progress_bar(filled: usize, total: usize) -> Style {
    if filled == total {
        status_ready()
    } else if filled > 0 {
        accent()
    } else {
        text_dim()
    }
}

pub fn splash_art() -> Style {
    Style::default().fg(Color::Rgb(205, 210, 220))
}

pub fn splash_art_dim() -> Style {
    Style::default().fg(Color::Rgb(110, 115, 128))
}

pub fn mascot_art() -> Style {
    Style::default().fg(Color::Rgb(185, 190, 200))
}

pub fn accent() -> Style {
    Style::default().fg(get_accent_color())
}

pub fn accent_bold() -> Style {
    Style::default()
        .fg(get_accent_color())
        .add_modifier(Modifier::BOLD)
}

pub fn text() -> Style {
    Style::default().fg(get_text_primary())
}

pub fn text_bold() -> Style {
    Style::default()
        .fg(get_text_primary())
        .add_modifier(Modifier::BOLD)
}

pub fn text_muted() -> Style {
    Style::default().fg(get_text_secondary())
}

pub fn text_dim() -> Style {
    Style::default().fg(get_text_muted())
}

pub fn border() -> Style {
    if is_light_theme() {
        Style::default().fg(Color::Rgb(180, 180, 200))
    } else {
        Style::default().fg(Color::Rgb(40, 60, 70))
    }
}

pub fn border_focused() -> Style {
    Style::default().fg(get_accent_color())
}

pub fn selected() -> Style {
    Style::default()
        .bg(get_accent_color())
        .fg(if is_light_theme() {
            Color::Rgb(255, 255, 255)
        } else {
            Color::Rgb(18, 18, 24)
        })
        .add_modifier(Modifier::BOLD)
}

pub fn status_ready() -> Style {
    Style::default().fg(SUCCESS)
}

pub fn status_warn() -> Style {
    status_accent()
}

pub fn status_error() -> Style {
    Style::default().fg(ERROR)
}

pub fn code_style() -> Style {
    Style::default().fg(get_accent_color())
}

pub fn system_detail_text() -> Style {
    Style::default().fg(get_text_secondary())
}

pub fn user_message_bg() -> Color {
    get_bg_surface()
}

pub fn user_message_text() -> Style {
    Style::default()
        .fg(get_text_primary())
        .bg(user_message_bg())
}

pub fn assistant_message_bg() -> Color {
    if is_light_theme() {
        Color::Rgb(238, 238, 246)
    } else {
        Color::Rgb(22, 22, 30)
    }
}

pub fn assistant_message_text() -> Style {
    Style::default()
        .fg(get_text_primary())
        .bg(assistant_message_bg())
}

pub fn todo_panel_label() -> Style {
    Style::default()
        .fg(get_accent_color())
        .bg(assistant_message_bg())
        .add_modifier(Modifier::BOLD)
}

pub fn todo_item_pending() -> Style {
    Style::default()
        .fg(get_text_muted())
        .bg(assistant_message_bg())
}

pub fn todo_item_active() -> Style {
    Style::default()
        .fg(get_accent_color())
        .bg(assistant_message_bg())
        .add_modifier(Modifier::BOLD)
}

pub fn todo_item_done() -> Style {
    Style::default()
        .fg(SUCCESS)
        .bg(assistant_message_bg())
}

pub fn todo_item_skipped() -> Style {
    Style::default()
        .fg(get_text_muted())
        .bg(assistant_message_bg())
}

pub fn clear_area(f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    f.render_widget(ratatui::widgets::Clear, area);
    if is_light_theme() {
        f.render_widget(
            ratatui::widgets::Block::default().style(Style::default().bg(get_bg_color())),
            area,
        );
    }
}
