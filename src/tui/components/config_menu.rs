use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::config;
use crate::tui::mascot;
use crate::tui::theme;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    theme::clear_area(f, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_focused())
        .title(" Playhouse - Settings (/config) ");

    let inner = block.inner(area);
    f.render_widget(block, area);
    mascot::render_corner(f, area, &app.workspace, 72);

    let sections = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .split(inner);

    let tab_labels = config::config_tab_labels();
    let tab_spans: Vec<Span> = tab_labels
        .iter()
        .enumerate()
        .flat_map(|(i, label)| {
            let style = if i == app.config_tab {
                theme::accent_bold()
            } else {
                theme::text_muted()
            };
            let prefix = if i == 0 { "  " } else { "  ·  " };
            vec![
                Span::styled(prefix, theme::text_dim()),
                Span::styled(*label, style),
            ]
        })
        .collect();
    f.render_widget(Paragraph::new(Line::from(tab_spans)), sections[0]);

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "  TUI settings for humans - agents use `playhouse config --json`",
            theme::text_muted(),
        ))),
        sections[1],
    );

    let items: Vec<ListItem> = app
        .config_options
        .iter()
        .enumerate()
        .map(|(i, (name, desc, enabled))| {
            let selected = i == app.config_selected;
            let marker = if selected { "› " } else { "  " };
            let status = if *enabled {
                Span::styled(" [*] ON  ", theme::status_ready())
            } else {
                Span::styled(" [ ] OFF ", theme::text_dim())
            };
            let name_style = if selected {
                theme::accent_bold()
            } else {
                theme::text()
            };
            ListItem::new(Line::from(vec![
                Span::styled(marker, name_style),
                status,
                Span::styled(name.as_str(), name_style),
                Span::styled(" - ", theme::text_dim()),
                Span::styled(desc.as_str(), theme::text_muted()),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.config_selected));
    f.render_stateful_widget(List::new(items), sections[2], &mut state);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  ↑/↓", theme::accent()),
            Span::styled(" navigate · ", theme::text_dim()),
            Span::styled("Space", theme::accent()),
            Span::styled(" toggle · ", theme::text_dim()),
            Span::styled("←/→", theme::accent()),
            Span::styled(" tab · ", theme::text_dim()),
            Span::styled("Esc", theme::accent()),
            Span::styled(" close", theme::text_dim()),
        ])),
        sections[3],
    );
}
