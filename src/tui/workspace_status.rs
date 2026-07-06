use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::tools::playhouse_dir;

#[derive(Debug, Clone, Default)]
pub struct WorkspaceStatus {
    pub last_stars: Option<u8>,
    pub last_grade: Option<String>,
    pub last_run: Option<String>,
}

#[derive(Deserialize)]
struct ScoreFile {
    #[serde(default, rename = "generatedAt")]
    generated_at: Option<String>,
    #[serde(default, rename = "playhouseScore")]
    playhouse_score: Option<ScoreBody>,
}

#[derive(Deserialize)]
struct ScoreBody {
    stars: u8,
    grade: String,
    #[allow(dead_code)]
    passed: bool,
}

pub fn load(workspace: &str) -> WorkspaceStatus {
    let path = score_path(workspace);
    let Ok(content) = fs::read_to_string(path) else {
        return WorkspaceStatus::default();
    };
    let Ok(file) = serde_json::from_str::<ScoreFile>(&content) else {
        return WorkspaceStatus::default();
    };
    let Some(score) = file.playhouse_score else {
        return WorkspaceStatus::default();
    };
    WorkspaceStatus {
        last_stars: Some(score.stars),
        last_grade: Some(score.grade),
        last_run: file
            .generated_at
            .as_deref()
            .and_then(format_generated_at),
    }
}

fn score_path(workspace: &str) -> PathBuf {
    playhouse_dir(workspace).join("reports").join("score.json")
}

fn format_generated_at(raw: &str) -> Option<String> {
    let secs = raw.parse::<u64>().ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(secs);
    let delta = now.saturating_sub(secs);
    if delta < 60 {
        Some("just now".into())
    } else if delta < 3600 {
        Some(format!("{}m ago", delta / 60))
    } else if delta < 86_400 {
        Some(format!("{}h ago", delta / 3600))
    } else {
        Some(format!("{}d ago", delta / 86_400))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_age_works() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(format_generated_at(&now.to_string()).as_deref(), Some("just now"));
    }
}
