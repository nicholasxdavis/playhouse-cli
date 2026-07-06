use std::path::{Path, PathBuf};

use crate::config::playhouse_home;

pub fn find_asset(workspace: &str, filename: &str) -> Option<PathBuf> {
    let candidates = [
        Path::new(workspace).join(filename),
        playhouse_home().join(filename),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

pub fn load_asset_lines(workspace: &str, filename: &str, embedded: &str) -> Vec<String> {
    if let Some(path) = find_asset(workspace, filename) {
        if let Ok(content) = std::fs::read_to_string(path) {
            let lines: Vec<String> = content.lines().map(str::to_string).collect();
            if !lines.is_empty() {
                return lines;
            }
        }
    }
    embedded.lines().map(str::to_string).collect()
}
