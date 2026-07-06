use std::path::Path;
use std::time::Instant;

use crate::tui::walk::{collect_files as walk_collect_files, MAX_FILES};

const MAX_FILTER_RESULTS: usize = 200;

#[derive(Clone, Debug)]
pub struct MentionEntry {
    pub path: String,
    pub(crate) lower_basename: String,
    lower_path: String,
    depth: u8,
}

#[derive(Debug)]
pub struct MentionIndex {
    entries: Vec<MentionEntry>,
    workspace: String,
    built_at: Option<Instant>,
}

impl MentionIndex {
    pub fn new(workspace: impl Into<String>) -> Self {
        Self {
            entries: Vec::new(),
            workspace: workspace.into(),
            built_at: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entry(&self, index: usize) -> Option<&MentionEntry> {
        self.entries.get(index)
    }

    pub fn spawn_refresh(&self) -> std::sync::mpsc::Receiver<Vec<MentionEntry>> {
        let (tx, rx) = std::sync::mpsc::channel();
        let workspace = self.workspace.clone();
        std::thread::spawn(move || {
            let root = Path::new(&workspace);
            let entries = if root.is_dir() {
                walk_collect_files(root)
                    .into_iter()
                    .take(MAX_FILES)
                    .map(|path| {
                        let lower_path = path.to_lowercase();
                        let lower_basename = basename(&lower_path).to_string();
                        let depth = path.matches('/').count() as u8;
                        MentionEntry {
                            path,
                            lower_path,
                            lower_basename,
                            depth,
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };
            let _ = tx.send(entries);
        });
        rx
    }

    pub fn apply_entries(&mut self, entries: Vec<MentionEntry>) {
        self.entries = entries;
        self.built_at = Some(Instant::now());
    }

    pub fn filter(&self, query: &str) -> Vec<usize> {
        let query_lower = query.to_lowercase();

        let mut ranked: Vec<(usize, u8, usize)> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| {
                rank_entry(entry, &query_lower).map(|rank| (i, rank, entry.depth as usize))
            })
            .collect();

        ranked.sort_by(|(i_a, rank_a, depth_a), (i_b, rank_b, depth_b)| {
            rank_a
                .cmp(rank_b)
                .then_with(|| depth_a.cmp(depth_b))
                .then_with(|| {
                    self.entries[*i_a]
                        .lower_basename
                        .len()
                        .cmp(&self.entries[*i_b].lower_basename.len())
                })
                .then_with(|| self.entries[*i_a].path.cmp(&self.entries[*i_b].path))
        });

        ranked
            .into_iter()
            .take(MAX_FILTER_RESULTS)
            .map(|(i, _, _)| i)
            .collect()
    }
}

fn rank_entry(entry: &MentionEntry, query_lower: &str) -> Option<u8> {
    if query_lower.is_empty() {
        return Some(0);
    }
    if entry.lower_basename.starts_with(query_lower) {
        Some(0)
    } else if entry.lower_basename.contains(query_lower) {
        Some(1)
    } else if entry.lower_path.contains(query_lower) {
        Some(2)
    } else {
        None
    }
}

fn basename(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn is_mention_boundary(text: &str, at_byte: usize) -> bool {
    if at_byte == 0 {
        return true;
    }
    text[..at_byte]
        .chars()
        .last()
        .map(|c| c.is_whitespace())
        .unwrap_or(true)
}

fn is_query_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | '\\')
}

pub fn active_mention_token(text: &str, cursor: usize) -> Option<(usize, &str)> {
    if cursor == 0 || cursor > text.len() {
        return None;
    }

    let before = &text[..cursor];
    let at_pos = before.rfind('@')?;

    if !is_mention_boundary(text, at_pos) {
        return None;
    }

    let query = &before[at_pos + 1..];
    if query.chars().any(|c| !is_query_char(c)) {
        return None;
    }

    Some((at_pos, query))
}

pub fn extract_mentions(text: &str) -> Vec<String> {
    let mut mentions = Vec::new();
    let mut i = 0;
    while i < text.len() {
        if let Some(rel) = text[i..].find('@') {
            let at_pos = i + rel;
            if !is_mention_boundary(text, at_pos) {
                i = at_pos + 1;
                continue;
            }

            let start = at_pos + 1;
            let rest = &text[start..];
            let end = rest
                .char_indices()
                .find(|(_, c)| !is_query_char(*c))
                .map(|(j, _)| j)
                .unwrap_or(rest.len());

            if end > 0 {
                mentions.push(text[start..start + end].to_string());
            }

            i = start + end;
        } else {
            break;
        }
    }
    mentions
}

pub fn parent_hint(path: &str) -> Option<&str> {
    let parent = path.rsplit_once('/')?.0;
    if parent.is_empty() {
        None
    } else {
        Some(parent)
    }
}
