use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::audit::{self, AuditProgress};
use crate::config::load_settings;
use crate::detect;
use crate::install;
use crate::score::PlayhouseScore;
use crate::tui::app::TaskKind;
use crate::tui::ui_blocks::{ContentBlock, TodoItem, TodoStatus};
use crate::types::CheckStatus;
use crate::workspace;

#[derive(Debug)]
pub enum TaskEvent {
    Started { label: String },
    Progress {
        label: String,
        blocks: Vec<ContentBlock>,
    },
    Finished {
        blocks: Vec<ContentBlock>,
        success: bool,
        summary: String,
        doctor_stats: Option<(usize, usize)>,
    },
}

pub fn spawn_task(
    kind: TaskKind,
    workspace_path: String,
    tx: mpsc::UnboundedSender<TaskEvent>,
) {
    tokio::spawn(async move {
        let label = task_label(&kind);
        let _ = tx.send(TaskEvent::Started { label: label.clone() });

        let (blocks, success, summary, doctor_stats) = match kind {
            TaskKind::Doctor => {
                let r = run_doctor(&workspace_path).await;
                (r.0, r.1, r.2, r.3)
            }
            TaskKind::Install => {
                let r = run_install(&workspace_path).await;
                (r.0, r.1, r.2, None)
            }
            TaskKind::Init { stay_on_track } => {
                let r = run_init(&workspace_path, stay_on_track).await;
                (r.0, r.1, r.2, None)
            }
            TaskKind::Verify => {
                let r = run_verify_task(&workspace_path, tx.clone()).await;
                (r.0, r.1, r.2, None)
            }
            TaskKind::Lighthouse { url } => {
                let r = run_lighthouse(&workspace_path, &url, tx.clone()).await;
                (r.0, r.1, r.2, None)
            }
            TaskKind::Playwright { pattern } => {
                let r = run_playwright(&workspace_path, pattern.as_deref(), tx.clone()).await;
                (r.0, r.1, r.2, None)
            }
            TaskKind::Trivy => {
                let r = run_trivy(&workspace_path, tx.clone()).await;
                (r.0, r.1, r.2, None)
            }
            TaskKind::Arkenar { url } => {
                let r = run_arkenar(&workspace_path, &url, tx.clone()).await;
                (r.0, r.1, r.2, None)
            }
        };

        let _ = tx.send(TaskEvent::Finished {
            blocks,
            success,
            summary,
            doctor_stats,
        });
    });
}

fn task_label(kind: &TaskKind) -> String {
    match kind {
        TaskKind::Doctor => "Checking tools…".into(),
        TaskKind::Install => "Installing Playwright, Trivy, Arkenar…".into(),
        TaskKind::Init { .. } => "Initializing workspace…".into(),
        TaskKind::Verify => "Verify · QA Suite".into(),
        TaskKind::Lighthouse { .. } => "Lighthouse audit…".into(),
        TaskKind::Playwright { .. } => "Playwright tests…".into(),
        TaskKind::Trivy => "Trivy security scan…".into(),
        TaskKind::Arkenar { .. } => "Arkenar DAST scan…".into(),
    }
}

fn send_progress(tx: &mpsc::UnboundedSender<TaskEvent>, label: &str, blocks: Vec<ContentBlock>) {
    let _ = tx.send(TaskEvent::Progress {
        label: label.to_string(),
        blocks,
    });
}

struct VerifyTracker {
    items: Vec<TodoItem>,
    labels: HashMap<&'static str, usize>,
}

impl VerifyTracker {
    fn new() -> Self {
        let steps: [(&'static str, &'static str); 7] = [
            ("prepare", "Prepare tools"),
            ("doctor", "Toolchain check"),
            ("trivy", "Trivy security"),
            ("functional", "Functional tests"),
            ("arkenar", "Arkenar DAST"),
            ("lighthouse", "Lighthouse audit"),
            ("score", "Playhouse Stars"),
        ];
        let mut labels = HashMap::new();
        let items: Vec<TodoItem> = steps
            .iter()
            .enumerate()
            .map(|(i, (id, label))| {
                labels.insert(*id, i);
                TodoItem {
                    text: (*label).into(),
                    status: if i == 0 {
                        TodoStatus::Active
                    } else {
                        TodoStatus::Pending
                    },
                    detail: None,
                }
            })
            .collect();
        Self { items, labels }
    }

    fn start(&mut self, id: &'static str, detail: &str) {
        if let Some(&idx) = self.labels.get(id) {
            for (i, item) in self.items.iter_mut().enumerate() {
                if i < idx && item.status == TodoStatus::Pending {
                    item.status = TodoStatus::Done;
                }
            }
            if let Some(item) = self.items.get_mut(idx) {
                item.status = TodoStatus::Active;
                item.detail = Some(detail.into());
            }
        }
    }

    fn done(&mut self, id: &'static str, detail: &str, skipped: bool, ok: bool) {
        if let Some(&idx) = self.labels.get(id) {
            if let Some(item) = self.items.get_mut(idx) {
                item.status = if skipped {
                    TodoStatus::Skipped
                } else if ok {
                    TodoStatus::Done
                } else {
                    TodoStatus::Done
                };
                item.detail = Some(detail.into());
            }
            if let Some(next) = self.items.get_mut(idx + 1) {
                if next.status == TodoStatus::Pending {
                    next.status = TodoStatus::Active;
                }
            }
        }
    }

    fn computing(&mut self, detail: &str) {
        for item in &mut self.items {
            if item.status == TodoStatus::Active {
                item.status = TodoStatus::Done;
            }
        }
        if let Some(item) = self.items.last_mut() {
            item.status = TodoStatus::Active;
            item.detail = Some(detail.into());
        }
    }

    fn finish(&mut self) {
        for item in &mut self.items {
            if item.status == TodoStatus::Active {
                item.status = TodoStatus::Done;
            }
        }
    }

    fn blocks(&self) -> Vec<ContentBlock> {
        vec![ContentBlock::todo_list("Verify · QA Suite", self.items.clone())]
    }
}

async fn run_verify_task(
    workspace: &str,
    tx: mpsc::UnboundedSender<TaskEvent>,
) -> (Vec<ContentBlock>, bool, String) {
    let settings = load_settings();
    let url = workspace::resolve_verify_url(workspace, &settings);
    let mut tracker = VerifyTracker::new();
    send_progress(&tx, "Verify · QA Suite", tracker.blocks());

    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<AuditProgress>();
    let ws = workspace.to_string();
    let url_clone = url.clone();
    let settings_clone = settings.clone();

    let audit_handle = tokio::spawn(async move {
        audit::run_audit_with_progress(
            &ws,
            url_clone.as_deref(),
            &settings_clone,
            true,
            Some(move |event| {
                let _ = progress_tx.send(event);
            }),
        )
        .await
    });

    while let Some(event) = progress_rx.recv().await {
        match event {
            AuditProgress::StepStart { id, label } => {
                tracker.start(id, &label);
                send_progress(&tx, &label, tracker.blocks());
            }
            AuditProgress::StepDone {
                id,
                detail,
                skipped,
                ok,
                ..
            } => {
                tracker.done(id, &detail, skipped, ok);
                send_progress(&tx, "Verify · QA Suite", tracker.blocks());
            }
            AuditProgress::Computing { label } => {
                tracker.computing(&label);
                send_progress(&tx, &label, tracker.blocks());
            }
        }
    }

    let report = audit_handle.await.unwrap();
    tracker.finish();

    let success = report.exit_code == 0;
    let score = &report.score;
    let summary = if success {
        format!("Verify passed — {} / 100 stars ({})", score.stars, score.grade)
    } else {
        format!(
            "Verify failed — {} / 100 stars ({}) · exit {}",
            score.stars, score.grade, report.exit_code
        )
    };

    let blocks = vec![
        ContentBlock::todo_list("Verify · QA Suite", tracker.items),
        ContentBlock::score_report(
            PlayhouseScore {
                stars: score.stars,
                grade: score.grade.clone(),
                grade_emoji: score.grade_emoji.clone(),
                passed: score.passed,
                categories: score.categories.clone(),
                why: score.why.clone(),
                methodology: score.methodology.clone(),
            },
            report.exit_code,
            report.engines.clone(),
        ),
    ];

    (blocks, success, summary)
}

async fn run_install(workspace: &str) -> (Vec<ContentBlock>, bool, String) {
    let report = install::ensure_all(workspace, true).await;
    let ok = report.errors.is_empty();
    let mut items = Vec::new();
    items.push(TodoItem {
        text: format!(
            "Trivy — {}",
            if report.trivy { "installed" } else { "failed" }
        ),
        status: if report.trivy {
            TodoStatus::Done
        } else {
            TodoStatus::Active
        },
        detail: None,
    });
    items.push(TodoItem {
        text: format!(
            "Playwright — {}",
            if report.playwright { "installed" } else { "failed" }
        ),
        status: if report.playwright {
            TodoStatus::Done
        } else {
            TodoStatus::Active
        },
        detail: None,
    });
    items.push(TodoItem {
        text: format!(
            "Lighthouse — {}",
            if report.lighthouse { "installed" } else { "failed" }
        ),
        status: if report.lighthouse {
            TodoStatus::Done
        } else {
            TodoStatus::Active
        },
        detail: None,
    });
    items.push(TodoItem {
        text: format!(
            "Arkenar — {}",
            if report.arkenar { "installed" } else { "failed" }
        ),
        status: if report.arkenar {
            TodoStatus::Done
        } else {
            TodoStatus::Active
        },
        detail: None,
    });
    let summary = if ok {
        "Install complete".into()
    } else {
        format!("Install had errors: {}", report.errors.join("; "))
    };
    let mut blocks = vec![ContentBlock::todo_list("Install · Bundled Tools", items)];
    for msg in report.messages {
        blocks.push(ContentBlock::text(msg));
    }
    for err in report.errors {
        blocks.push(ContentBlock::text(format!("Error: {err}")));
    }
    (blocks, ok, summary)
}

async fn run_init(workspace: &str, stay_on_track: bool) -> (Vec<ContentBlock>, bool, String) {
    let settings = load_settings();
    match workspace::init_workspace(workspace, &settings, true, stay_on_track, true).await {
        Ok(report) => {
            let summary = "Workspace initialized".into();
            let mut blocks = vec![ContentBlock::text(format!(
                "Created {}\nBrief: {}",
                report.playhouse_dir, report.brief_path
            ))];
            if let Some(skill) = report.skill_path {
                blocks.push(ContentBlock::text(format!("Stay-on-track: {skill}")));
            }
            if let Some(skill) = report.playhouse_skill_path {
                blocks.push(ContentBlock::text(format!("Playhouse agent skill: {skill}")));
            }
            (blocks, true, summary)
        }
        Err(e) => (
            vec![ContentBlock::text(format!("Init failed: {e}"))],
            false,
            e,
        ),
    }
}

async fn run_doctor(workspace: &str) -> (Vec<ContentBlock>, bool, String, Option<(usize, usize)>) {
    let settings = load_settings();
    if settings.auto_install_tools {
        let _ = install::ensure_all(workspace, true).await;
    }

    let checks = tokio::task::spawn_blocking({
        let ws = workspace.to_string();
        move || detect::run_doctor(&ws)
    })
    .await
    .unwrap_or_default();

    let pass = checks.iter().filter(|c| c.status == CheckStatus::Pass).count();
    let total = checks.len();
    let items: Vec<TodoItem> = checks
        .iter()
        .map(|c| {
            let status = match c.status {
                CheckStatus::Pass => TodoStatus::Done,
                CheckStatus::Warn => TodoStatus::Pending,
                CheckStatus::Fail => TodoStatus::Active,
            };
            TodoItem {
                text: format!("{} — {}", c.name, c.detail),
                status,
                detail: None,
            }
        })
        .collect();
    let ok = checks.iter().all(|c| c.status != CheckStatus::Fail);
    let summary = if ok {
        "Doctor complete — no hard failures".into()
    } else {
        "Doctor found failures".into()
    };
    (
        vec![ContentBlock::todo_list("Doctor · Tool Health", items)],
        ok,
        summary,
        Some((pass, total)),
    )
}

async fn run_lighthouse(
    workspace: &str,
    url: &str,
    tx: mpsc::UnboundedSender<TaskEvent>,
) -> (Vec<ContentBlock>, bool, String) {
    send_progress(
        &tx,
        "Lighthouse audit…",
        vec![ContentBlock::tool_running("Lighthouse", &format!("Auditing {url}"))],
    );
    let code = crate::engines::lighthouse::run(url, workspace, true, true).await;
    let success = code == 0;
    let summary = if success {
        format!("Lighthouse passed for {url}")
    } else {
        format!("Lighthouse failed for {url} (exit {code})")
    };
    (
        vec![
            ContentBlock::tool_done("Lighthouse", &summary, success),
            ContentBlock::text("Run `playhouse lighthouse --json` for score details."),
        ],
        success,
        summary,
    )
}

async fn run_playwright(
    workspace: &str,
    pattern: Option<&str>,
    tx: mpsc::UnboundedSender<TaskEvent>,
) -> (Vec<ContentBlock>, bool, String) {
    send_progress(
        &tx,
        "Playwright tests…",
        vec![ContentBlock::tool_running("Playwright", "Running functional tests")],
    );
    let code = crate::engines::playwright::run(workspace, pattern, true, true).await;
    let success = code == 0;
    let summary = if success {
        "Playwright tests passed".into()
    } else {
        format!("Playwright failed (exit {code})")
    };
    (
        vec![
            ContentBlock::tool_done("Playwright", &summary, success),
            ContentBlock::text("Run `playhouse playwright --json` for full report."),
        ],
        success,
        summary,
    )
}

async fn run_arkenar(
    workspace: &str,
    url: &str,
    tx: mpsc::UnboundedSender<TaskEvent>,
) -> (Vec<ContentBlock>, bool, String) {
    send_progress(
        &tx,
        "Arkenar DAST scan…",
        vec![ContentBlock::tool_running("Arkenar", &format!("Scanning {url}"))],
    );
    let code = crate::engines::arkenar::run(url, workspace, true, true).await;
    let success = code == 0;
    let summary = if success {
        format!("Arkenar scan clean for {url}")
    } else if code == 3 {
        format!("Arkenar found high/medium issues for {url}")
    } else {
        format!("Arkenar failed for {url} (exit {code})")
    };
    (
        vec![
            ContentBlock::tool_done("Arkenar", &summary, success),
            ContentBlock::text("Report: .playhouse/reports/arkenar.json · `playhouse arkenar --json`"),
        ],
        success,
        summary,
    )
}

async fn run_trivy(workspace: &str, tx: mpsc::UnboundedSender<TaskEvent>) -> (Vec<ContentBlock>, bool, String) {
    send_progress(
        &tx,
        "Trivy security scan…",
        vec![ContentBlock::tool_running("Trivy", "Scanning filesystem and secrets")],
    );
    let code = crate::engines::trivy::run(workspace, true, true).await;
    let success = code == 0;
    let summary = if success {
        "Trivy scan clean".into()
    } else {
        format!("Trivy findings detected (exit {code})")
    };
    (
        vec![
            ContentBlock::tool_done("Trivy", &summary, success),
            ContentBlock::text("Run `playhouse trivy --json` for full findings."),
        ],
        success,
        summary,
    )
}
