use std::fs;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::audit::AuditProgress;
use crate::tools;

const STEPS: &[&str] = &["prepare", "doctor", "trivy", "functional", "arkenar", "lighthouse"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProgressState {
    pub active: bool,
    pub started_at: String,
    pub elapsed_secs: u64,
    pub current_step: Option<String>,
    pub current_label: Option<String>,
    pub steps_total: u32,
    pub steps_completed: u32,
    pub percent_complete: u32,
}

pub fn progress_path(workspace: &str) -> std::path::PathBuf {
    tools::playhouse_dir(workspace)
        .join("reports")
        .join("verify-progress.json")
}

pub fn read_state(workspace: &str) -> Option<VerifyProgressState> {
    let path = progress_path(workspace);
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn status_json(workspace: &str) -> Value {
    match read_state(workspace) {
        Some(state) if state.active => json!({
            "command": "status",
            "active": true,
            "progress": state,
            "progressPath": progress_path(workspace),
        }),
        Some(state) => json!({
            "command": "status",
            "active": false,
            "lastProgress": state,
            "progressPath": progress_path(workspace),
        }),
        None => json!({
            "command": "status",
            "active": false,
            "progressPath": progress_path(workspace),
        }),
    }
}

pub struct Tracker {
    workspace: String,
    started: Instant,
    steps_completed: u32,
    current_step: Option<String>,
    current_label: Option<String>,
    finished: bool,
}

impl Tracker {
    pub fn start(workspace: &str) -> Self {
        let tracker = Self {
            workspace: workspace.to_string(),
            started: Instant::now(),
            steps_completed: 0,
            current_step: None,
            current_label: None,
            finished: false,
        };
        tracker.persist(true);
        tracker
    }

    pub fn handle(&mut self, event: &AuditProgress) {
        match event {
            AuditProgress::StepStart { id, label } => {
                self.current_step = Some((*id).to_string());
                self.current_label = Some(label.clone());
            }
            AuditProgress::StepDone { id, .. } => {
                self.steps_completed = self
                    .steps_completed
                    .max(step_index(id).map(|i| i as u32 + 1).unwrap_or(self.steps_completed));
                self.current_step = Some((*id).to_string());
            }
            AuditProgress::Computing { label } => {
                self.current_label = Some(label.clone());
            }
        }
        self.persist(true);
    }

    pub fn finish(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        self.persist(false);
        let _ = fs::remove_file(progress_path(&self.workspace));
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        self.finish();
    }
}

impl Tracker {
    fn persist(&self, active: bool) {
        let reports = tools::playhouse_dir(&self.workspace).join("reports");
        let _ = fs::create_dir_all(&reports);
        let elapsed = self.started.elapsed();
        let steps_total = STEPS.len() as u32;
        let percent = if steps_total == 0 {
            0
        } else {
            ((self.steps_completed as f64 / steps_total as f64) * 100.0).round() as u32
        };
        let state = VerifyProgressState {
            active,
            started_at: format_elapsed_start(self.started),
            elapsed_secs: elapsed.as_secs(),
            current_step: self.current_step.clone(),
            current_label: self.current_label.clone(),
            steps_total,
            steps_completed: self.steps_completed,
            percent_complete: percent.min(100),
        };
        if let Ok(json) = serde_json::to_string_pretty(&state) {
            let _ = fs::write(progress_path(&self.workspace), json);
        }
    }
}

fn step_index(id: &str) -> Option<usize> {
    STEPS.iter().position(|s| *s == id)
}

fn format_elapsed_start(started: Instant) -> String {
    let secs = started.elapsed().as_secs();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    (now.saturating_sub(secs)).to_string()
}

pub fn emit_step_stderr(event: &AuditProgress) {
    let line = match event {
        AuditProgress::StepStart { id, label } => format!("[verify] start {id}: {label}"),
        AuditProgress::StepDone {
            id,
            ok,
            skipped,
            ..
        } => {
            let status = if *skipped {
                "skipped"
            } else if *ok {
                "ok"
            } else {
                "fail"
            };
            format!("[verify] done {id}: {status}")
        }
        AuditProgress::Computing { label } => format!("[verify] {label}"),
    };
    eprintln!("{line}");
}
