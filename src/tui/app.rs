use std::path::{Path, PathBuf};

use ratatui::layout::Rect;
use ratatui::widgets::ListState;

use crate::detect;
use crate::install::InstallProfile;
use crate::tui::config::{self, PlayhouseSettings};
use crate::tui::mention::{self, MentionEntry, MentionIndex};
use crate::tui::selection::{CharRange, LineSelection};
use crate::tui::ui_blocks::ContentBlock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SlashCategory {
    Qa,
    Agent,
    Config,
    Meta,
}

impl SlashCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Qa => "QA",
            Self::Agent => "Agent",
            Self::Config => "Config",
            Self::Meta => "Meta",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SlashCommand {
    pub command: String,
    pub description: String,
    pub category: SlashCategory,
}

pub fn verify_flag_help_lines() -> [&'static str; 4] {
    [
        "/verify [URL] [--test PAT] [--start-server CMD] [--port N] [--server-timeout SEC]",
        "  Headless: playhouse verify [--url URL] [--test PATTERN] [--start-server CMD]",
        "            [--server-port N] [--server-timeout SEC] --json",
        "  Example: /verify http://localhost:3000 --start-server \"npm run dev\" --port 3000",
    ]
}

pub fn default_slash_commands() -> Vec<SlashCommand> {
    let cmds: [(&str, &str, SlashCategory); 26] = [
        ("/doctor", "Tool health (/doctor resolve rebuilds native deps)", SlashCategory::Qa),
        ("/install", "Install bundled tools (/install --minimal | --full)", SlashCategory::Qa),
        ("/init", "Initialize .playhouse/ (--no-skill, --stay-on-track)", SlashCategory::Qa),
        ("/verify", "Full QA (/verify [url] --test pat --start-server cmd)", SlashCategory::Qa),
        ("/score", "Star audit (/score last = saved; /score [url] = audit only)", SlashCategory::Qa),
        ("/functional", "Run detected functional test runner", SlashCategory::Qa),
        ("/lighthouse", "Performance & accessibility audit", SlashCategory::Qa),
        ("/playwright", "Browser E2E tests (Playwright only)", SlashCategory::Qa),
        ("/trivy", "Security & secret scan", SlashCategory::Qa),
        ("/arkenar", "DAST web security scan", SlashCategory::Qa),
        ("/status", "Verify progress (when verify is running)", SlashCategory::Qa),
        ("/test", "Test baseplates (/test list|init|add|run)", SlashCategory::Qa),
        ("/agents", "Show workspace brief", SlashCategory::Agent),
        ("/agent", "Agent manifest (/agent status, handoff, plan, …)", SlashCategory::Agent),
        ("/skill", "Agent skill (/skill enable|install, disable, status)", SlashCategory::Agent),
        ("/export", "Export .playhouse/BRIEF.md", SlashCategory::Agent),
        ("/stay-on-track", "Stay-on-track skill (enable, disable, status)", SlashCategory::Agent),
        ("/config", "Settings & preferences (/config set KEY VALUE)", SlashCategory::Config),
        ("/auth", "Audit auth (/auth login --token … [--url URL])", SlashCategory::Config),
        ("/uninstall", "Remove bundled tools (/uninstall --yes [--global])", SlashCategory::Config),
        ("/version", "Show Playhouse CLI version and install source", SlashCategory::Meta),
        ("/upgrade", "Check GitHub and npm for updates", SlashCategory::Meta),
        ("/update", "Apply latest Playhouse release", SlashCategory::Meta),
        ("/help", "Interactive command reference", SlashCategory::Meta),
        ("/clear", "Clear the feed", SlashCategory::Meta),
        ("/quit", "Exit Playhouse", SlashCategory::Meta),
    ];
    cmds.into_iter()
        .map(|(c, d, category)| SlashCommand {
            command: c.into(),
            description: d.into(),
            category,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedRole {
    User,
    System,
}

#[derive(Debug, Clone)]
pub struct FeedEntry {
    pub role: FeedRole,
    pub blocks: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    SlashMenu,
    MentionMenu,
    HelpMenu,
    ConfigMenu,
}

#[derive(Debug, Clone, Default)]
pub struct VerifyParams {
    pub url: Option<String>,
    pub test_pattern: Option<String>,
    pub start_server: Option<String>,
    pub server_port: Option<u16>,
    pub server_timeout: u64,
}

impl VerifyParams {
    pub fn new() -> Self {
        Self {
            server_timeout: 120,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum TaskKind {
    Doctor { resolve: bool },
    Install { profile: InstallProfile },
    Init {
        stay_on_track: bool,
        no_skill: bool,
    },
    Verify { params: VerifyParams },
    Score { url: Option<String> },
    Lighthouse { url: String },
    Playwright { pattern: Option<String> },
    Functional { pattern: Option<String> },
    Trivy,
    Arkenar { url: String },
    Handoff { params: VerifyParams },
}

pub struct App {
    pub running: bool,
    pub mode: AppMode,
    pub workspace: String,
    pub local_server: Option<String>,
    pub settings: PlayhouseSettings,
    pub config_options: Vec<(String, String, bool)>,
    pub config_tab: usize,
    pub config_selected: usize,
    pub input_text: String,
    pub cursor_position: usize,
    pub input_select_anchor: Option<usize>,
    pub input_pane_area: Rect,
    pub tick_count: u64,
    pub feed: Vec<FeedEntry>,
    pub feed_scroll_pos: f32,
    pub feed_scroll_target: f32,
    pub feed_stick_bottom: bool,
    pub feed_scroll_dragging: bool,
    pub feed_line_count: usize,
    pub feed_viewport_lines: usize,
    pub feed_pane_area: Rect,
    pub feed_scrollbar_area: Rect,
    pub feed_scrollbar_track_area: Rect,
    pub feed_text_selection: Option<LineSelection>,
    pub feed_select_dragging: bool,
    pub busy: bool,
    pub busy_label: String,
    pub task_feed_idx: Option<usize>,
    pub slash_commands: Vec<SlashCommand>,
    pub slash_filter: String,
    pub slash_selected: usize,
    pub help_tab: usize,
    pub help_selected: usize,
    pub help_list_state: ListState,
    pub advisories: Vec<String>,
    pub tools_summary: String,
    pub mention_index: MentionIndex,
    pub mention_filtered: Vec<usize>,
    pub mention_selected: usize,
    pub mention_refresh_rx: Option<std::sync::mpsc::Receiver<Vec<MentionEntry>>>,
    pub doctor_pass_count: Option<(usize, usize)>,
    brief_cache: String,
    brief_cache_valid: bool,
}

impl App {
    pub fn new(workspace: &str, after_splash: bool) -> Self {
        let settings = config::load_settings();
        config::apply_theme_from_settings(&settings);
        let local_server = detect::find_local_server(workspace);
        let config_options = config::config_options_for_tab(0, workspace, &settings);

        let mut app = Self {
            running: true,
            mode: AppMode::Normal,
            workspace: workspace.to_string(),
            local_server,
            settings,
            config_options,
            config_tab: 0,
            config_selected: 0,
            input_text: String::new(),
            cursor_position: 0,
            input_select_anchor: None,
            input_pane_area: Rect::default(),
            tick_count: 0,
            feed: Vec::new(),
            feed_scroll_pos: 0.0,
            feed_scroll_target: 0.0,
            feed_stick_bottom: true,
            feed_scroll_dragging: false,
            feed_line_count: 0,
            feed_viewport_lines: 0,
            feed_pane_area: Rect::default(),
            feed_scrollbar_area: Rect::default(),
            feed_scrollbar_track_area: Rect::default(),
            feed_text_selection: None,
            feed_select_dragging: false,
            busy: false,
            busy_label: String::new(),
            task_feed_idx: None,
            slash_commands: default_slash_commands(),
            slash_filter: String::new(),
            slash_selected: 0,
            help_tab: 0,
            help_selected: 0,
            help_list_state: ListState::default(),
            advisories: load_advisories(workspace),
            tools_summary: "run /doctor to check tools".into(),
            mention_index: MentionIndex::new(workspace),
            mention_filtered: Vec::new(),
            mention_selected: 0,
            mention_refresh_rx: None,
            doctor_pass_count: None,
            brief_cache: String::new(),
            brief_cache_valid: false,
        };

        app.push_system(&welcome_message(&app, after_splash));
        app
    }

    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
        self.animate_feed_scroll();
        self.poll_mention_refresh();
    }

    pub fn needs_animation_frame(&self) -> bool {
        if self.is_busy() {
            return true;
        }
        if self.has_revealing_content() {
            return true;
        }
        if self.feed_scroll_dragging {
            return true;
        }
        !self.feed_stick_bottom && (self.feed_scroll_target - self.feed_scroll_pos).abs() >= 0.4
    }

    fn has_revealing_content(&self) -> bool {
        for entry in self.feed.iter().rev().take(2) {
            for block in &entry.blocks {
                if let ContentBlock::ScoreReport {
                    reveal_tick,
                    score,
                    ..
                } = block
                {
                    if *reveal_tick == 0 {
                        continue;
                    }
                    let elapsed = self.tick_count.saturating_sub(*reveal_tick);
                    let needed = 20 + score.why.len() as u64 * 8;
                    if elapsed < needed {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn update_last_system_blocks(&mut self, blocks: Vec<ContentBlock>) {
        self.feed_stick_bottom = true;
        if let Some(idx) = self.task_feed_idx {
            if let Some(entry) = self.feed.get_mut(idx) {
                entry.blocks = blocks;
                return;
            }
        }
        self.task_feed_idx = Some(self.feed.len());
        self.feed.push(FeedEntry {
            role: FeedRole::System,
            blocks,
        });
    }

    /// Replace the active task feed entry with final blocks (spinner → + in place).
    pub fn finish_task_feed(&mut self, blocks: Vec<ContentBlock>) {
        self.feed_stick_bottom = true;
        if let Some(idx) = self.task_feed_idx.take() {
            if let Some(entry) = self.feed.get_mut(idx) {
                entry.blocks = blocks;
                return;
            }
        }
        self.feed.push(FeedEntry {
            role: FeedRole::System,
            blocks,
        });
    }

    pub fn clear_task_feed(&mut self) {
        self.task_feed_idx = None;
    }

    pub fn on_resize(&mut self) {
        self.feed_scroll_dragging = false;
        self.feed_select_dragging = false;
        self.feed_text_selection = None;
        self.input_select_anchor = None;
        self.input_pane_area = Rect::default();
        self.feed_pane_area = Rect::default();
        self.feed_scrollbar_area = Rect::default();
        self.feed_scrollbar_track_area = Rect::default();
        self.refresh_feed_scroll_metrics();
        if self.feed_stick_bottom {
            self.scroll_feed_bottom();
        } else {
            self.clamp_feed_scroll(self.feed_max_scroll_top());
        }
    }

    /// Slow network probe — never call from resize hot path.
    pub fn refresh_local_server(&mut self) {
        self.local_server = detect::find_local_server(&self.workspace);
    }

    pub fn set_doctor_stats(&mut self, pass: usize, total: usize) {
        self.doctor_pass_count = Some((pass, total));
        self.tools_summary = format!("{pass}/{total} tools ready");
        self.refresh_local_server();
        self.invalidate_brief();
    }

    pub fn invalidate_brief(&mut self) {
        self.brief_cache_valid = false;
    }

    pub fn brief_text(&mut self) -> &str {
        if !self.brief_cache_valid {
            self.brief_cache = self.build_brief();
            self.brief_cache_valid = true;
        }
        &self.brief_cache
    }

    pub fn save_settings(&mut self) {
        self.settings.last_workspace = Some(self.workspace.clone());
        config::save_settings(&self.settings);
        self.config_options =
            config::config_options_for_tab(self.config_tab, &self.workspace, &self.settings);
    }

    pub fn refresh_config_options(&mut self) {
        self.config_options =
            config::config_options_for_tab(self.config_tab, &self.workspace, &self.settings);
        if self.config_selected >= self.config_options.len() {
            self.config_selected = self.config_options.len().saturating_sub(1);
        }
    }

    fn poll_mention_refresh(&mut self) {
        let Some(rx) = &self.mention_refresh_rx else {
            return;
        };
        if let Ok(entries) = rx.try_recv() {
            self.mention_index.apply_entries(entries);
            self.mention_refresh_rx = None;
            self.rebuild_mention_filter();
        }
    }

    pub fn is_busy(&self) -> bool {
        self.busy
    }

    pub fn feed_scroll_line(&self) -> usize {
        self.feed_scroll_pos.round() as usize
    }

    pub fn feed_max_scroll_top(&self) -> usize {
        self.feed_line_count.saturating_sub(self.feed_viewport_lines)
    }

    pub fn feed_scrolled_up(&self) -> bool {
        !self.feed_stick_bottom
    }

    pub fn refresh_feed_scroll_metrics(&mut self) {
        if self.feed_pane_area.height == 0 {
            return;
        }
        let m = crate::tui::components::feed::measure_feed(self, self.feed_pane_area);
        self.feed_line_count = m.total_lines;
        self.feed_viewport_lines = m.viewport_lines;
        self.clamp_feed_scroll(m.max_scroll_top);
    }

    pub fn clamp_feed_scroll(&mut self, max_top: usize) {
        let max = max_top as f32;
        if self.feed_stick_bottom {
            self.feed_scroll_target = max;
            self.feed_scroll_pos = max;
        } else {
            self.feed_scroll_target = self.feed_scroll_target.min(max);
            self.feed_scroll_pos = self.feed_scroll_pos.min(max);
        }
    }

    fn animate_feed_scroll(&mut self) {
        if self.feed_scroll_dragging {
            return;
        }
        if self.feed_stick_bottom {
            return;
        }
        let diff = self.feed_scroll_target - self.feed_scroll_pos;
        if diff.abs() < 0.4 {
            self.feed_scroll_pos = self.feed_scroll_target;
        } else {
            let step = (diff.abs() * 0.38).clamp(1.0, 10.0);
            self.feed_scroll_pos += diff.signum() * step;
        }
        let max = self.feed_max_scroll_top() as f32;
        if self.feed_scroll_pos >= max - 0.5 {
            self.feed_stick_bottom = true;
        }
    }

    pub fn scroll_feed_up(&mut self, lines: u16) {
        self.refresh_feed_scroll_metrics();
        self.feed_stick_bottom = false;
        self.feed_scroll_dragging = false;
        let step = lines.max(1) as f32;
        self.feed_scroll_target = (self.feed_scroll_pos - step).max(0.0);
    }

    pub fn scroll_feed_down(&mut self, lines: u16) {
        self.refresh_feed_scroll_metrics();
        self.feed_scroll_dragging = false;
        let step = lines.max(1) as f32;
        let max = self.feed_max_scroll_top() as f32;
        self.feed_scroll_target = (self.feed_scroll_pos + step).min(max);
        self.feed_stick_bottom = self.feed_scroll_target >= max - 0.5;
    }

    pub fn scroll_feed_page_up(&mut self) {
        let lines = (self.feed_viewport_lines.saturating_mul(85) / 100).max(4) as u16;
        self.scroll_feed_up(lines);
    }

    pub fn scroll_feed_page_down(&mut self) {
        let lines = (self.feed_viewport_lines.saturating_mul(85) / 100).max(4) as u16;
        self.scroll_feed_down(lines);
    }

    pub fn scroll_feed_bottom(&mut self) {
        self.feed_stick_bottom = true;
        self.feed_scroll_dragging = false;
        self.refresh_feed_scroll_metrics();
        let max = self.feed_max_scroll_top() as f32;
        self.feed_scroll_target = max;
        self.feed_scroll_pos = max;
    }

    pub fn scroll_feed_top(&mut self) {
        self.feed_stick_bottom = false;
        self.feed_scroll_dragging = false;
        self.feed_scroll_target = 0.0;
        self.feed_scroll_pos = 0.0;
    }

    pub fn handle_mouse(&mut self, event: crossterm::event::MouseEvent) {
        use crossterm::event::MouseEventKind;
        use ratatui::layout::Position;

        let pos = Position::new(event.column, event.row);
        let on_input = self.input_pane_area.contains(pos);

        if on_input {
            self.feed_select_dragging = false;
            self.feed_scroll_dragging = false;
            self.feed_text_selection = None;
            if matches!(event.kind, MouseEventKind::Down(_)) {
                self.input_select_anchor = None;
            }
        }

        if self.mode == AppMode::Normal {
            self.handle_feed_mouse(event);
        }
    }

    pub fn handle_feed_mouse(&mut self, event: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};
        use ratatui::layout::Position;

        self.refresh_feed_scroll_metrics();
        let max = self.feed_max_scroll_top();
        let pos = Position::new(event.column, event.row);
        let on_scrollbar = self.feed_scrollbar_area.contains(pos);
        let on_feed = self.feed_pane_area.contains(pos);
        let scroll_y = self.feed_scroll_line();

        if let Some((line, col)) = crate::tui::components::feed::feed_pos_to_line_col(
            self.feed_pane_area,
            scroll_y,
            event.row,
            event.column,
        ) {
            match event.kind {
                MouseEventKind::Down(MouseButton::Left) if on_feed && !on_scrollbar => {
                    self.feed_select_dragging = true;
                    self.input_select_anchor = None;
                    self.feed_text_selection =
                        Some(LineSelection::new(line, col, line, col));
                }
                MouseEventKind::Drag(MouseButton::Left) if self.feed_select_dragging => {
                    if let Some(sel) = &mut self.feed_text_selection {
                        *sel = LineSelection::new(sel.start_line, sel.start_col, line, col);
                    }
                }
                MouseEventKind::Up(MouseButton::Left) if self.feed_select_dragging => {
                    self.feed_select_dragging = false;
                }
                _ => {}
            }
        }

        if max == 0 {
            return;
        }

        match event.kind {
            MouseEventKind::ScrollUp if on_feed || on_scrollbar => self.scroll_feed_up(1),
            MouseEventKind::ScrollDown if on_feed || on_scrollbar => self.scroll_feed_down(1),
            MouseEventKind::Down(MouseButton::Left) if on_scrollbar => {
                self.feed_stick_bottom = false;
                if let Some(layout) = crate::tui::components::feed::scrollbar_layout(
                    self.feed_scrollbar_area,
                    self.feed_line_count,
                    self.feed_viewport_lines,
                    self.feed_scroll_pos,
                ) {
                    if layout.track_area.contains(pos) {
                        let rel = event.row as f32 - layout.track_area.y as f32;
                        let thumb_end = layout.thumb_top + layout.thumb_height;
                        if rel < layout.thumb_top {
                            self.scroll_feed_page_up();
                        } else if rel >= thumb_end {
                            self.scroll_feed_page_down();
                        } else {
                            self.feed_scroll_dragging = true;
                            self.set_scroll_from_track_row(event.row);
                        }
                    }
                }
            }
            MouseEventKind::Drag(MouseButton::Left) if self.feed_scroll_dragging => {
                self.set_scroll_from_track_row(event.row);
            }
            MouseEventKind::Up(MouseButton::Left) if self.feed_scroll_dragging => {
                self.feed_scroll_dragging = false;
                if self.feed_scroll_pos >= max as f32 - 0.5 {
                    self.feed_stick_bottom = true;
                    self.feed_scroll_target = max as f32;
                }
            }
            _ => {}
        }
    }

    fn set_scroll_from_track_row(&mut self, row: u16) {
        let Some(layout) = crate::tui::components::feed::scrollbar_layout(
            self.feed_scrollbar_area,
            self.feed_line_count,
            self.feed_viewport_lines,
            self.feed_scroll_pos,
        ) else {
            return;
        };
        let track = layout.track_area;
        if track.height == 0 {
            return;
        }
        let rel = (row.saturating_sub(track.y) as f32).clamp(0.0, track.height as f32 - 0.001);
        let travel = (track.height as f32 - layout.thumb_height).max(0.001);
        let frac = rel / travel;
        self.feed_scroll_target = (frac * layout.max_scroll).clamp(0.0, layout.max_scroll);
        self.feed_scroll_pos = self.feed_scroll_target;
    }

    pub fn push_user(&mut self, text: &str) {
        self.feed_stick_bottom = true;
        self.feed.push(FeedEntry {
            role: FeedRole::User,
            blocks: vec![ContentBlock::text(text)],
        });
    }

    pub fn push_system(&mut self, text: &str) {
        self.feed_stick_bottom = true;
        self.feed.push(FeedEntry {
            role: FeedRole::System,
            blocks: vec![ContentBlock::text(text)],
        });
    }

    pub fn push_blocks(&mut self, role: FeedRole, blocks: Vec<ContentBlock>) {
        self.feed_stick_bottom = true;
        self.feed.push(FeedEntry { role, blocks });
    }

    pub fn push_advisory(&mut self, text: &str) {
        self.advisories.push(text.to_string());
        save_advisory(&self.workspace, text);
        self.push_user(text);
        self.push_system("Note saved to .playhouse/advisories.log");
        self.invalidate_brief();
    }

    pub fn filtered_slash_commands(&self) -> Vec<&SlashCommand> {
        let filter = self.slash_filter.to_lowercase();
        self.slash_commands
            .iter()
            .filter(|c| {
                filter.is_empty()
                    || c.command.to_lowercase().contains(&filter)
                    || c.description.to_lowercase().contains(&filter)
            })
            .collect()
    }

    pub fn sync_slash_selection(&mut self) {
        let stem = self
            .input_text
            .trim_start_matches("./")
            .trim_start_matches('/')
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_lowercase();
        let filtered = self.filtered_slash_commands();
        if stem.is_empty() {
            self.slash_selected = 0;
            return;
        }
        if let Some(i) = filtered.iter().position(|c| {
            c.command
                .trim_start_matches('/')
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_lowercase()
                .starts_with(&stem)
        }) {
            self.slash_selected = i;
        }
    }

    pub fn active_mention_token(&self) -> Option<(usize, &str)> {
        let byte = self
            .input_text
            .char_indices()
            .nth(self.cursor_position)
            .map(|(i, _)| i)
            .unwrap_or(self.input_text.len());
        mention::active_mention_token(&self.input_text, byte)
    }

    pub fn mention_menu_entries(&self) -> impl Iterator<Item = &MentionEntry> + '_ {
        self.mention_filtered
            .iter()
            .filter_map(|&i| self.mention_index.entry(i))
    }

    pub fn mention_show_empty_state(&self) -> bool {
        self.mode == AppMode::MentionMenu
            && self.mention_filtered.is_empty()
            && self.active_mention_token().is_some()
    }

    fn update_mention_mode(&mut self) {
        if self.mode == AppMode::SlashMenu {
            return;
        }
        if self.active_mention_token().is_some() {
            if self.mode == AppMode::Normal {
                if self.mention_index.is_empty() && self.mention_refresh_rx.is_none() {
                    self.mention_refresh_rx = Some(self.mention_index.spawn_refresh());
                }
                self.mode = AppMode::MentionMenu;
                self.mention_selected = 0;
            }
            self.rebuild_mention_filter();
        } else if self.mode == AppMode::MentionMenu {
            self.mode = AppMode::Normal;
            self.mention_filtered.clear();
        }
    }

    fn rebuild_mention_filter(&mut self) {
        let query = self
            .active_mention_token()
            .map(|(_, q)| q.to_string())
            .unwrap_or_default();
        self.mention_filtered = self.mention_index.filter(&query);
        self.sync_mention_selection();
    }

    fn sync_mention_selection(&mut self) {
        if let Some((_, query)) = self.active_mention_token() {
            let q = query.to_lowercase();
            if let Some(i) = self
                .mention_filtered
                .iter()
                .position(|&idx| {
                    self.mention_index
                        .entry(idx)
                        .map(|e| e.lower_basename == q)
                        .unwrap_or(false)
                })
            {
                self.mention_selected = i;
                return;
            }
        }
        if self.mention_selected >= self.mention_filtered.len() {
            self.mention_selected = self.mention_filtered.len().saturating_sub(1);
        }
    }

    pub fn mention_menu_up(&mut self) {
        if self.mention_selected > 0 {
            self.mention_selected -= 1;
        }
    }

    pub fn mention_menu_down(&mut self) {
        if self.mention_selected + 1 < self.mention_filtered.len() {
            self.mention_selected += 1;
        }
    }

    pub fn mention_menu_select(&mut self) {
        let Some((start, query)) = self.active_mention_token().map(|(s, q)| (s, q.to_string()))
        else {
            return;
        };
        let path = self
            .mention_filtered
            .iter()
            .find_map(|&idx| {
                self.mention_index.entry(idx).and_then(|e| {
                    if e.lower_basename == query.to_lowercase() {
                        Some(e.path.clone())
                    } else {
                        None
                    }
                })
            })
            .or_else(|| {
                self.mention_filtered
                    .get(self.mention_selected)
                    .and_then(|&idx| self.mention_index.entry(idx).map(|e| e.path.clone()))
            });
        let Some(path) = path else {
            return;
        };
        let replacement = format!("@{path}");
        let end_byte = self
            .input_text
            .char_indices()
            .nth(self.cursor_position)
            .map(|(i, _)| i)
            .unwrap_or(self.input_text.len());
        self.input_text.replace_range(start..end_byte, &replacement);
        self.cursor_position = start + replacement.chars().count();
        self.mode = AppMode::Normal;
        self.mention_filtered.clear();
    }

    pub fn mention_menu_complete(&mut self) {
        if !self.mention_filtered.is_empty() {
            self.mention_menu_select();
        }
    }

    pub fn input_selection(&self) -> Option<CharRange> {
        self.input_select_anchor
            .map(|a| CharRange::new(a, self.cursor_position))
            .filter(|r| !r.is_empty())
    }

    pub fn clear_text_selection(&mut self) {
        self.input_select_anchor = None;
        self.feed_text_selection = None;
        self.feed_select_dragging = false;
    }

    pub fn select_all_input(&mut self) {
        if self.input_text.is_empty() {
            return;
        }
        self.input_select_anchor = Some(0);
        self.cursor_position = self.input_text.chars().count();
        self.feed_text_selection = None;
    }

    pub fn insert_char(&mut self, c: char) {
        if self.input_select_anchor.is_some() {
            self.delete_selection();
        }
        let byte_idx = char_byte_index(&self.input_text, self.cursor_position);
        self.input_text.insert(byte_idx, c);
        self.cursor_position += 1;
        self.clear_feed_selection();

        let is_slash = self.input_text.starts_with('/');
        if self.mode == AppMode::Normal && is_slash {
            self.mode = AppMode::SlashMenu;
            self.slash_selected = 0;
        }
        if self.mode == AppMode::SlashMenu {
            self.slash_filter = self.input_text.trim_start_matches('/').to_string();
            self.sync_slash_selection();
        }
        if self.mode == AppMode::SlashMenu && !is_slash {
            self.mode = AppMode::Normal;
        }
        self.update_mention_mode();
    }

    pub fn handle_paste(&mut self, text: &str) {
        let sanitized = text.replace('\r', "");
        if sanitized.is_empty() {
            return;
        }
        match self.mode {
            AppMode::Normal | AppMode::SlashMenu | AppMode::MentionMenu => {
                if self.input_select_anchor.is_some() {
                    self.delete_selection();
                }
                let byte_idx = char_byte_index(&self.input_text, self.cursor_position);
                self.input_text.insert_str(byte_idx, &sanitized);
                self.cursor_position += sanitized.chars().count();
                self.clear_feed_selection();
                let is_slash = self.input_text.starts_with('/');
                if self.mode == AppMode::Normal && is_slash {
                    self.mode = AppMode::SlashMenu;
                    self.slash_selected = 0;
                }
                if self.mode == AppMode::SlashMenu {
                    self.slash_filter = self.input_text.trim_start_matches('/').to_string();
                    self.sync_slash_selection();
                }
                if self.mode == AppMode::SlashMenu && !is_slash {
                    self.mode = AppMode::Normal;
                }
                self.update_mention_mode();
            }
            _ => {}
        }
    }

    pub fn delete_char(&mut self) {
        if self.input_select_anchor.is_some() {
            self.delete_selection();
            return;
        }
        if self.cursor_position == 0 {
            return;
        }
        let byte_idx = char_byte_index(&self.input_text, self.cursor_position);
        let prev = char_byte_index(&self.input_text, self.cursor_position - 1);
        self.input_text.replace_range(prev..byte_idx, "");
        self.cursor_position -= 1;
        if self.mode == AppMode::SlashMenu {
            self.slash_filter = self.input_text.trim_start_matches('/').to_string();
            if !self.input_text.starts_with('/') {
                self.mode = AppMode::Normal;
            }
        }
        self.update_mention_mode();
    }

    pub fn delete_forward(&mut self) {
        let len = self.input_text.chars().count();
        if self.cursor_position >= len {
            return;
        }
        let start = char_byte_index(&self.input_text, self.cursor_position);
        let end = char_byte_index(&self.input_text, self.cursor_position + 1);
        self.input_text.replace_range(start..end, "");
        self.update_mention_mode();
    }

    pub fn move_cursor_left(&mut self, extend: bool) {
        if self.cursor_position == 0 {
            return;
        }
        if extend {
            if self.input_select_anchor.is_none() {
                self.input_select_anchor = Some(self.cursor_position);
            }
        } else {
            self.input_select_anchor = None;
        }
        self.cursor_position -= 1;
    }

    pub fn move_cursor_right(&mut self, extend: bool) {
        let len = self.input_text.chars().count();
        if self.cursor_position >= len {
            return;
        }
        if extend {
            if self.input_select_anchor.is_none() {
                self.input_select_anchor = Some(self.cursor_position);
            }
        } else {
            self.input_select_anchor = None;
        }
        self.cursor_position += 1;
    }

    pub fn delete_selection(&mut self) {
        let Some(range) = self.input_selection() else {
            return;
        };
        let start = char_byte_index(&self.input_text, range.start);
        let end = char_byte_index(&self.input_text, range.end);
        self.input_text.replace_range(start..end, "");
        self.cursor_position = range.start.min(self.input_text.chars().count());
        self.input_select_anchor = None;
        self.update_mention_mode();
    }

    fn clear_feed_selection(&mut self) {
        self.feed_text_selection = None;
        self.feed_select_dragging = false;
    }

    pub fn copy_selection(&mut self) -> bool {
        if let Some(sel) = self.feed_text_selection {
            let lines = crate::tui::components::feed::build_feed_plain_lines(self, self.feed_pane_area);
            let text = sel.extract(&lines);
            if !text.is_empty() {
                return crate::tui::clipboard::write_text(&text);
            }
        }
        if let Some(range) = self.input_selection() {
            let text = range.slice(&self.input_text);
            if !text.is_empty() {
                return crate::tui::clipboard::write_text(&text);
            }
        }
        false
    }

    pub fn agent_brief(&self) -> String {
        self.build_brief()
    }

    fn build_brief(&self) -> String {
        let ws = crate::workspace::load_workspace_config(&self.workspace);
        let mut brief = crate::agent::build_brief_text(&self.workspace, &self.settings, &ws);

        if !self.advisories.is_empty() {
            brief.push_str("\n## Notes\n");
            for (i, note) in self.advisories.iter().enumerate() {
                brief.push_str(&format!("{}. {}\n", i + 1, note));
            }
        }

        let mentions: Vec<_> = self
            .feed
            .iter()
            .filter_map(|e| {
                if e.role == FeedRole::User {
                    e.blocks.first().and_then(|b| {
                        if let ContentBlock::Text { content } = b {
                            Some(content.clone())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            })
            .flat_map(|m| mention::extract_mentions(&m))
            .collect();

        if !mentions.is_empty() {
            brief.push_str("\n## Referenced Files (@mentions)\n");
            for p in mentions {
                brief.push_str(&format!("- @{p}\n"));
            }
        }

        brief
    }

    pub fn export_brief(&self) -> std::io::Result<PathBuf> {
        let dir = Path::new(&self.workspace).join(".playhouse");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("BRIEF.md");
        std::fs::write(&path, self.build_brief())?;
        Ok(path)
    }

    pub fn toggle_config(&mut self) {
        let was_stay = self.settings.stay_on_track_enabled;
        let was_ph = self.settings.playhouse_skill_enabled;
        config::toggle_config_option(
            &mut self.settings,
            self.config_tab,
            self.config_selected,
        );
        if self.config_tab == 2 && self.config_selected == 0 {
            if self.settings.playhouse_skill_enabled && !was_ph {
                match crate::workspace::install_playhouse_skill(&self.workspace, &self.settings) {
                    Ok(p) => self.push_system(&format!("Playhouse agent skill: {}", p.display())),
                    Err(e) => self.push_system(&format!("Playhouse skill error: {e}")),
                }
            } else if !self.settings.playhouse_skill_enabled && was_ph {
                let _ = crate::workspace::disable_playhouse_skill_mode(&self.workspace);
                self.push_system("Playhouse agent skill disabled for workspace");
            }
        }
        if self.config_tab == 4 && self.config_selected == 0 {
            if self.settings.stay_on_track_enabled && !was_stay {
                match crate::workspace::enable_stay_on_track_mode(&self.workspace, &self.settings) {
                    Ok(p) => self.push_system(&format!("Stay-on-track skill: {}", p.display())),
                    Err(e) => self.push_system(&format!("Stay-on-track error: {e}")),
                }
            } else if !self.settings.stay_on_track_enabled && was_stay {
                let _ = crate::workspace::disable_stay_on_track_mode(&self.workspace);
                self.push_system("Stay-on-track disabled for workspace");
            }
        }
        self.refresh_config_options();
        self.invalidate_brief();
    }
}

fn char_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn welcome_message(app: &App, after_splash: bool) -> String {
    let ws_cfg = crate::workspace::load_workspace_config(&app.workspace);
    let init_hint = if !ws_cfg.initialized {
        "\nRun /init to set up .playhouse/ in this workspace."
    } else {
        ""
    };

    if after_splash {
        return format!(
            "Workspace: {}\n\
             /help · / slash · @ files · Enter for notes\n\
             Agents: `playhouse agent --json`{init_hint}",
            app.workspace
        );
    }

    let ph = if app.settings.playhouse_skill_enabled {
        ".playhouse skill ON"
    } else {
        ".playhouse skill off"
    };
    let sot = if app.settings.stay_on_track_enabled {
        "stay-on-track ON"
    } else {
        "stay-on-track off"
    };
    format!(
        "Playhouse QA CLI - TUI for humans · headless for agents\n\
         Workspace: {}\n\
         /install · /doctor · /verify · /skill · /help · {} · {}\n\
         Agents: `playhouse agent --json`{init_hint}",
        app.workspace, ph, sot
    )
}

fn playhouse_dir(workspace: &str) -> PathBuf {
    Path::new(workspace).join(".playhouse")
}

fn load_advisories(workspace: &str) -> Vec<String> {
    let path = playhouse_dir(workspace).join("advisories.log");
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(|line| line.split_once(" | ").map(|(_, note)| note.to_string()))
        .collect()
}

fn save_advisory(workspace: &str, text: &str) {
    let dir = playhouse_dir(workspace);
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("advisories.log");
    let ts = chrono_lite_timestamp();
    let line = format!("{ts} | {text}\n");
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = f.write_all(line.as_bytes());
    }
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}
