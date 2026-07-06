use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
    Frame,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::tui::app::App;
use crate::tui::mention;
use crate::tui::text_box;
use crate::tui::theme;

const MAX_VISIBLE: usize = 8;

fn popup_area(area: Rect, height: u16, max_width: u16) -> Rect {
    let popup_x = area.x.saturating_add(1);
    let popup_width = area.width.saturating_sub(2).min(max_width).max(1);
    let right_limit = area.x.saturating_add(area.width);
    let width = popup_width.min(right_limit.saturating_sub(popup_x));
    let popup_y = if area.height > height.saturating_add(3) {
        area.y + area.height - height - 3
    } else {
        area.y
    };
    Rect {
        x: popup_x,
        y: popup_y,
        width,
        height,
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    if app.mode != crate::tui::app::AppMode::MentionMenu {
        return;
    }

    let entries: Vec<_> = app.mention_menu_entries().collect();
    let show_empty = app.mention_show_empty_state();

    if entries.is_empty() && !show_empty {
        return;
    }

    let visible_count = if show_empty {
        1
    } else {
        entries.len().min(MAX_VISIBLE)
    };
    let popup_height = visible_count as u16 + 1;
    let popup_area = popup_area(area, popup_height, 88);

    theme::clear_area(f, popup_area);

    let items: Vec<ListItem> = if show_empty {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("No matching files", theme::text_muted()),
            Span::styled(" - keep typing or Esc", theme::text_dim()),
        ]))]
    } else {
        entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let is_selected = i == app.mention_selected;
                let marker = if is_selected { "› " } else { "  " };
                let display = format!("@{}", entry.path);
                let name_style = if is_selected {
                    theme::accent_bold()
                } else {
                    theme::accent()
                };
                let desc_style = if is_selected {
                    theme::text_bold()
                } else {
                    theme::text_muted()
                };
                let name_col_width = 28usize.min(popup_area.width as usize / 2);
                let name_display = pad_or_truncate(&display, name_col_width);
                let hint = mention::parent_hint(&entry.path)
                    .map(|p| format!("in {p}"))
                    .unwrap_or_else(|| "workspace root".to_string());
                let hint_display = truncate_to_width(
                    &hint,
                    (popup_area.width as usize).saturating_sub(text_box::width(&format!(
                        "{marker}{name_display}  "
                    ))),
                );
                ListItem::new(Line::from(vec![
                    Span::styled(marker, name_style),
                    Span::styled(name_display, name_style),
                    Span::styled("  ", Style::default()),
                    Span::styled(hint_display, desc_style),
                ]))
            })
            .collect()
    };

    let list_area = Rect {
        x: popup_area.x,
        y: popup_area.y,
        width: popup_area.width,
        height: popup_area.height.saturating_sub(1),
    };
    let list = List::new(items);
    let mut state = ListState::default();
    if !show_empty {
        state.select(Some(app.mention_selected));
    }
    f.render_stateful_widget(list, list_area, &mut state);

    let footer_area = Rect {
        x: popup_area.x,
        y: popup_area.y + popup_area.height.saturating_sub(1),
        width: popup_area.width,
        height: 1,
    };
    let total = app.mention_filtered.len();
    let more = if total > MAX_VISIBLE {
        format!(" · +{} more", total - MAX_VISIBLE)
    } else {
        String::new()
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  filter · ", theme::text_dim()),
            Span::styled("↑/↓", theme::accent()),
            Span::styled(" · ", theme::text_dim()),
            Span::styled("Tab/Enter", theme::accent()),
            Span::styled(" select · ", theme::text_dim()),
            Span::styled("Esc", theme::accent()),
            Span::styled(format!(" cancel{more}"), theme::text_dim()),
        ])),
        footer_area,
    );
}

fn pad_or_truncate(text: &str, width: usize) -> String {
    let display_width = text.width();
    if display_width > width {
        truncate_to_width(text, width.saturating_sub(1))
    } else {
        format!("{text}{}", " ".repeat(width.saturating_sub(display_width)))
    }
}

fn truncate_to_width(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut w = 0;
    for ch in text.chars() {
        let cw = ch.width().unwrap_or(0);
        if w + cw > max_width {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out.push('…');
    out
}
