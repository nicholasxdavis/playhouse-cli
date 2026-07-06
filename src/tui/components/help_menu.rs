use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::theme;

pub fn render(f: &mut Frame, area: Rect, app: &mut App) {
    theme::clear_area(f, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_focused())
        .title(" Playhouse - Reference (/help) ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    let sections = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .split(inner);

    let tab_labels = ["Commands", "@ Files", "Workspace brief", "Headless CLI"];
    let tab_spans: Vec<Span> = tab_labels
        .iter()
        .enumerate()
        .flat_map(|(i, label)| {
            let style = if i == app.help_tab {
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

    match app.help_tab {
        0 => render_commands(f, sections[1], sections[2], sections[3], app),
        1 => render_mentions(f, sections[1], sections[2], sections[3]),
        2 => render_brief(f, sections[1], sections[2], sections[3], app),
        _ => render_engines(f, sections[1], sections[2], sections[3]),
    }
}

fn render_commands(f: &mut Frame, sub: Rect, body: Rect, footer: Rect, app: &mut App) {
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "  Browse slash commands - Enter runs the selected command:",
            theme::text_muted(),
        ))),
        sub,
    );

    let items: Vec<ListItem> = app
        .slash_commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let selected = i == app.help_selected;
            let marker = if selected { "› " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::styled(marker, if selected { theme::accent() } else { theme::text_dim() }),
                Span::styled(
                    &cmd.command,
                    if selected { theme::selected() } else { theme::accent() },
                ),
                Span::styled(" - ", theme::text_dim()),
                Span::styled(&cmd.description, theme::text_muted()),
            ]))
        })
        .collect();

    let mut state = std::mem::take(&mut app.help_list_state);
    state.select(Some(app.help_selected));
    f.render_stateful_widget(List::new(items), body, &mut state);
    app.help_list_state = state;

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  ↑/↓", theme::accent()),
            Span::styled(" navigate · ", theme::text_dim()),
            Span::styled("Enter", theme::accent()),
            Span::styled(" run · ", theme::text_dim()),
            Span::styled("←/→ Tab", theme::accent()),
            Span::styled(" switch tab · ", theme::text_dim()),
            Span::styled("Esc", theme::accent()),
            Span::styled(" close", theme::text_dim()),
        ])),
        footer,
    );
}

fn render_mentions(f: &mut Frame, sub: Rect, body: Rect, footer: Rect) {
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "  Attach workspace files with @ mentions in the input box:",
            theme::text_muted(),
        ))),
        sub,
    );

    let lines = vec![
        Line::from(vec![
            Span::styled("  Type ", theme::text_dim()),
            Span::styled("@", theme::accent_bold()),
            Span::styled(" at a word boundary to open the file picker", theme::text()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Example: ", theme::text_muted()),
            Span::styled("Check @src/main.rs for regressions", theme::text()),
        ]),
        Line::from(vec![
            Span::styled("  Example: ", theme::text_muted()),
            Span::styled("Run playwright on @tests/e2e/login.spec.ts", theme::text()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑/↓", theme::accent()),
            Span::styled(" filter · ", theme::text_dim()),
            Span::styled("Tab/Enter", theme::accent()),
            Span::styled(" complete · ", theme::text_dim()),
            Span::styled("Esc", theme::accent()),
            Span::styled(" dismiss", theme::text_dim()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  @ paths are included in ", theme::text_dim()),
            Span::styled("/export", theme::accent()),
            Span::styled(" and ", theme::text_dim()),
            Span::styled("/agents", theme::accent()),
            Span::styled(" output.", theme::text_dim()),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), body);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  ←/→ Tab", theme::accent()),
            Span::styled(" switch section · ", theme::text_dim()),
            Span::styled("Esc", theme::accent()),
            Span::styled(" close", theme::text_dim()),
        ])),
        footer,
    );
}

fn render_brief(f: &mut Frame, sub: Rect, body: Rect, footer: Rect, app: &mut App) {
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "  Workspace context for external tools - view with /agents, export with /export:",
            theme::text_muted(),
        ))),
        sub,
    );

    let brief = app.brief_text();
    let preview: String = brief.lines().take(24).collect::<Vec<_>>().join("\n");
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("  {preview}"),
            theme::code_style(),
        ))),
        body,
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  /export", theme::accent()),
            Span::styled(" writes .playhouse/BRIEF.md · ", theme::text_dim()),
            Span::styled("Esc", theme::accent()),
            Span::styled(" close", theme::text_dim()),
        ])),
        footer,
    );
}

fn render_engines(f: &mut Frame, sub: Rect, body: Rect, footer: Rect) {
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "  Agents invoke Playhouse headless - same commands a human runs in shell/CI:",
            theme::text_muted(),
        ))),
        sub,
    );

    let lines = vec![
        Line::from(vec![Span::styled("  playhouse agent --json", theme::accent_bold()), Span::styled(" - START HERE: full manifest", theme::text())]),
        Line::from(vec![Span::styled("  playhouse skill install", theme::accent_bold()), Span::styled(" - .playhouse/SKILL.md for agents (recommended)", theme::text())]),
        Line::from(vec![Span::styled("  playhouse agent status --json", theme::accent()), Span::styled(" - quick health and next actions", theme::text())]),
        Line::from(vec![Span::styled("  playhouse agent handoff --json", theme::accent()), Span::styled(" - verify + handoff bundle", theme::text())]),
        Line::from(vec![Span::styled("  playhouse config schema --json", theme::accent()), Span::styled(" - settable config keys", theme::text())]),
        Line::from(vec![Span::styled("  playhouse install", theme::accent()), Span::styled(" - auto-install Playwright + Trivy", theme::text())]),
        Line::from(vec![Span::styled("  playhouse init [--stay-on-track]", theme::accent()), Span::styled(" - set up .playhouse/", theme::text())]),
        Line::from(vec![Span::styled("  playhouse doctor --json", theme::accent()), Span::styled(" - tool health", theme::text())]),
        Line::from(vec![Span::styled("  playhouse verify [--url URL] --json", theme::accent()), Span::styled(" - full QA suite + Playhouse Stars", theme::text())]),
        Line::from(vec![Span::styled("  playhouse score [--url URL] [--last] --json", theme::accent()), Span::styled(" - 0-100 star rating audit", theme::text())]),
        Line::from(vec![Span::styled("  playhouse playwright [pattern] --json", theme::accent()), Span::styled(" - functional tests", theme::text())]),
        Line::from(vec![Span::styled("  playhouse arkenar [url] --json", theme::accent()), Span::styled(" - DAST web scan (replaces ZAP)", theme::text())]),
        Line::from(vec![Span::styled("  playhouse trivy --json", theme::accent()), Span::styled(" - security + secrets", theme::text())]),
        Line::from(vec![Span::styled("  playhouse lighthouse [url] --json", theme::accent()), Span::styled(" - performance audit", theme::text())]),
        Line::from(vec![Span::styled("  playhouse stay-on-track enable", theme::accent()), Span::styled(" - optional .playhouse/stay-on-track/SKILL.md", theme::text())]),
        Line::from(vec![Span::styled("  playhouse export", theme::accent()), Span::styled(" - write .playhouse/BRIEF.md", theme::text())]),
        Line::from(vec![Span::styled("  playhouse config --json", theme::accent()), Span::styled(" - all settings", theme::text())]),
        Line::from(""),
        Line::from(vec![Span::styled("  Exit codes: ", theme::text_muted()), Span::styled("0=pass 1=fail 3=arkenar 4=trivy 5=run playhouse install", theme::text())]),
    ];
    f.render_widget(Paragraph::new(lines), body);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  ←/→ Tab", theme::accent()),
            Span::styled(" switch section · ", theme::text_dim()),
            Span::styled("Esc", theme::accent()),
            Span::styled(" close", theme::text_dim()),
        ])),
        footer,
    );
}
