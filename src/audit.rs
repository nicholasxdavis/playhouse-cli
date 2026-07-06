use crate::config::PlayhouseSettings;
use crate::detect;
use crate::engines;
use crate::install;
use crate::project::{self, FunctionalRunner, ProjectProfile};
use crate::score::{self, EngineResult, PlayhouseScore};
use crate::workspace;

#[derive(Clone, Debug)]
pub enum AuditProgress {
    StepStart { id: &'static str, label: String },
    StepDone {
        id: &'static str,
        label: String,
        ok: bool,
        detail: String,
        skipped: bool,
    },
    Computing { label: String },
}

pub async fn run_audit(
    workspace: &str,
    url: Option<&str>,
    settings: &PlayhouseSettings,
    quiet: bool,
) -> AuditReport {
    run_audit_with_progress(workspace, url, settings, quiet, None::<fn(AuditProgress)>).await
}

pub async fn run_audit_with_progress<F>(
    workspace: &str,
    url: Option<&str>,
    settings: &PlayhouseSettings,
    quiet: bool,
    mut on_progress: Option<F>,
) -> AuditReport
where
    F: FnMut(AuditProgress),
{
    let mut progress = |event: AuditProgress| {
        if let Some(ref mut cb) = on_progress {
            cb(event);
        }
    };

    let profile = project::detect(workspace);

    progress(AuditProgress::StepStart {
        id: "prepare",
        label: "Preparing tools…".into(),
    });
    if settings.auto_install_tools {
        let install_report = install::ensure_profile(
            workspace,
            profile.install_profile(),
            true,
        )
        .await;
        let install_ok = install_report.ok();
        let detail = if install_ok {
            format!("Profile: {}", install_report.profile)
        } else {
            install_report.errors.join("; ")
        };
        progress(AuditProgress::StepDone {
            id: "prepare",
            label: "Tools ready".into(),
            ok: install_ok,
            detail,
            skipped: false,
        });
        if !install_ok {
            let doctor = detect::run_doctor(workspace);
            let playhouse_score = score::compute(&[], Some(&doctor), settings);
            return AuditReport {
                exit_code: 5,
                score: playhouse_score,
                engines: vec![],
                doctor,
            };
        }
    } else {
        progress(AuditProgress::StepDone {
            id: "prepare",
            label: "Tools ready".into(),
            ok: true,
            detail: "Auto-install disabled".into(),
            skipped: false,
        });
    }

    progress(AuditProgress::StepStart {
        id: "doctor",
        label: "Checking toolchain…".into(),
    });
    let doctor = detect::run_doctor(workspace);
    let doctor_fail = doctor.iter().any(|c| c.status == crate::types::CheckStatus::Fail);
    let doctor_pass = doctor.iter().filter(|c| c.status == crate::types::CheckStatus::Pass).count();
    progress(AuditProgress::StepDone {
        id: "doctor",
        label: "Toolchain".into(),
        ok: !doctor_fail,
        detail: format!("{doctor_pass}/{} tools ready", doctor.len()),
        skipped: false,
    });

    if doctor_fail {
        progress(AuditProgress::Computing {
            label: "Toolchain failed — skipping engines".into(),
        });
        let playhouse_score = score::compute(&[], Some(&doctor), settings);
        return AuditReport {
            exit_code: 5,
            score: playhouse_score,
            engines: vec![],
            doctor,
        };
    }

    let target_url = url
        .map(String::from)
        .or_else(|| workspace::resolve_verify_url(workspace, settings));

    let mut engines = Vec::new();

    if settings.skip_trivy_in_verify {
        engines.push(score::skipped("trivy"));
        progress(AuditProgress::StepDone {
            id: "trivy",
            label: "Trivy".into(),
            ok: true,
            detail: "Skipped in settings".into(),
            skipped: true,
        });
    } else {
        progress(AuditProgress::StepStart {
            id: "trivy",
            label: "Running Trivy security scan…".into(),
        });
        let (code, metrics) = engines::trivy::execute(workspace).await;
        let vulns = metrics
            .get("summary")
            .and_then(|s| s.get("vulnerabilities"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let secrets = metrics
            .get("summary")
            .and_then(|s| s.get("secrets"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        engines.push(EngineResult {
            engine: "trivy".into(),
            exit_code: code,
            skipped: false,
            metrics,
        });
        progress(AuditProgress::StepDone {
            id: "trivy",
            label: "Trivy".into(),
            ok: code == 0,
            detail: format!("{vulns} vulns, {secrets} secrets"),
            skipped: false,
        });
    }

    if settings.skip_playwright_in_verify {
        engines.push(score::skipped("functional"));
        progress(AuditProgress::StepDone {
            id: "functional",
            label: "Functional tests".into(),
            ok: true,
            detail: "Skipped in settings".into(),
            skipped: true,
        });
    } else if profile.functional_runner == FunctionalRunner::None {
        engines.push(score::skipped("functional"));
        progress(AuditProgress::StepDone {
            id: "functional",
            label: "Functional tests".into(),
            ok: true,
            detail: "No runner for this stack".into(),
            skipped: true,
        });
    } else {
        let runner_label = profile.functional_runner.as_str();
        progress(AuditProgress::StepStart {
            id: "functional",
            label: format!("Running {runner_label} tests…"),
        });
        let (code, metrics) = engines::functional::execute(workspace, &profile, None).await;
        let (passed, failed, skipped, no_tests) = functional_stats(&metrics);
        engines.push(EngineResult {
            engine: "functional".into(),
            exit_code: code,
            skipped: false,
            metrics,
        });
        progress(AuditProgress::StepDone {
            id: "functional",
            label: "Functional tests".into(),
            ok: code == 0,
            detail: if no_tests {
                "No tests ran".into()
            } else {
                format!("{passed} passed, {failed} failed, {skipped} skipped ({runner_label})")
            },
            skipped: false,
        });
    }

    run_browser_audits(
        workspace,
        &profile,
        target_url.as_deref(),
        settings,
        &mut engines,
        &mut progress,
    )
    .await;

    progress(AuditProgress::Computing {
        label: "Computing Playhouse Stars…".into(),
    });
    let playhouse_score = score::compute(&engines, Some(&doctor), settings);
    let _ = score::save_report(workspace, &playhouse_score, &engines);

    let worst_code = engines
        .iter()
        .filter(|e| !e.skipped)
        .map(|e| e.exit_code)
        .max()
        .unwrap_or(0);

    let exit_code = if !playhouse_score.passed {
        worst_code.max(1)
    } else {
        0
    };

    if !quiet {
        print_summary(&playhouse_score, &engines);
    }

    AuditReport {
        exit_code,
        score: playhouse_score,
        engines,
        doctor,
    }
}

async fn run_browser_audits<F>(
    workspace: &str,
    profile: &ProjectProfile,
    target_url: Option<&str>,
    settings: &PlayhouseSettings,
    engines: &mut Vec<EngineResult>,
    progress: &mut F,
) where
    F: FnMut(AuditProgress),
{
    if !profile.browser_audits {
        let reason = format!("not-applicable: browser audits N/A for {} stack", profile.stack.as_str());
        engines.push(score::skipped_reason("arkenar", &reason));
        engines.push(score::skipped_reason("lighthouse", &reason));
        progress(AuditProgress::StepDone {
            id: "arkenar",
            label: "Arkenar DAST".into(),
            ok: true,
            detail: "N/A for this stack".into(),
            skipped: true,
        });
        progress(AuditProgress::StepDone {
            id: "lighthouse",
            label: "Lighthouse".into(),
            ok: true,
            detail: "N/A for this stack".into(),
            skipped: true,
        });
        return;
    }

    let Some(target) = target_url else {
        let reason = "no-url: set playhouse config set default_url or start dev server";
        engines.push(score::skipped_reason("arkenar", reason));
        engines.push(score::skipped_reason("lighthouse", reason));
        progress(AuditProgress::StepDone {
            id: "arkenar",
            label: "Arkenar DAST".into(),
            ok: true,
            detail: "No URL — skipped".into(),
            skipped: true,
        });
        progress(AuditProgress::StepDone {
            id: "lighthouse",
            label: "Lighthouse".into(),
            ok: true,
            detail: "No URL — skipped".into(),
            skipped: true,
        });
        return;
    };

    if settings.skip_arkenar_in_verify {
        engines.push(score::skipped_reason("arkenar", "settings: skip_arkenar_in_verify"));
        progress(AuditProgress::StepDone {
            id: "arkenar",
            label: "Arkenar DAST".into(),
            ok: true,
            detail: "Skipped in settings".into(),
            skipped: true,
        });
    } else {
        progress(AuditProgress::StepStart {
            id: "arkenar",
            label: format!("Running Arkenar DAST on {target}…"),
        });
        let (code, metrics) = engines::arkenar::execute(target, workspace).await;
        let summary = metrics.get("summary");
        let high = summary.and_then(|s| s.get("high")).and_then(|v| v.as_u64()).unwrap_or(0);
        let medium = summary.and_then(|s| s.get("medium")).and_then(|v| v.as_u64()).unwrap_or(0);
        let low = summary.and_then(|s| s.get("low")).and_then(|v| v.as_u64()).unwrap_or(0);
        engines.push(EngineResult {
            engine: "arkenar".into(),
            exit_code: code,
            skipped: false,
            metrics,
        });
        progress(AuditProgress::StepDone {
            id: "arkenar",
            label: "Arkenar DAST".into(),
            ok: code == 0,
            detail: format!("high={high} medium={medium} low={low}"),
            skipped: false,
        });
    }

    if settings.skip_lighthouse_in_verify {
        engines.push(score::skipped_reason(
            "lighthouse",
            "settings: skip_lighthouse_in_verify",
        ));
        progress(AuditProgress::StepDone {
            id: "lighthouse",
            label: "Lighthouse".into(),
            ok: true,
            detail: "Skipped in settings".into(),
            skipped: true,
        });
    } else {
        progress(AuditProgress::StepStart {
            id: "lighthouse",
            label: format!("Running Lighthouse on {target}…"),
        });
        let (code, metrics) = engines::lighthouse::execute(target, workspace, settings).await;
        let lh_detail = lighthouse_detail(&metrics);
        engines.push(EngineResult {
            engine: "lighthouse".into(),
            exit_code: code,
            skipped: false,
            metrics,
        });
        progress(AuditProgress::StepDone {
            id: "lighthouse",
            label: "Lighthouse".into(),
            ok: code == 0,
            detail: lh_detail,
            skipped: false,
        });
    }
}

fn lighthouse_detail(metrics: &serde_json::Value) -> String {
    let scores = &metrics["scores"];
    let perf = scores["performance"].as_f64().map(|v| (v * 100.0).round() as u32);
    let a11y = scores["accessibility"].as_f64().map(|v| (v * 100.0).round() as u32);
    let bp = scores["bestPractices"].as_f64().map(|v| (v * 100.0).round() as u32);
    let seo = scores["seo"].as_f64().map(|v| (v * 100.0).round() as u32);
    match (perf, a11y, bp, seo) {
        (Some(p), Some(a), Some(b), Some(s)) => format!("perf {p} a11y {a} bp {b} seo {s}"),
        _ => "scores pending".into(),
    }
}

#[derive(Debug, Clone)]
pub struct AuditReport {
    pub exit_code: i32,
    pub score: PlayhouseScore,
    pub engines: Vec<EngineResult>,
    pub doctor: Vec<crate::types::HealthCheck>,
}

fn functional_stats(metrics: &serde_json::Value) -> (u64, u64, u64, bool) {
    let no_tests = metrics.get("noTests").and_then(|v| v.as_bool()).unwrap_or(false);
    let stats = metrics.get("stats");
    let passed = stats
        .and_then(|s| s.get("passed"))
        .and_then(|v| v.as_u64())
        .or_else(|| {
            stats
                .and_then(|s| s.get("expected"))
                .and_then(|v| v.as_u64())
        })
        .unwrap_or(0);
    let failed = stats
        .and_then(|s| s.get("failed"))
        .and_then(|v| v.as_u64())
        .or_else(|| {
            stats
                .and_then(|s| s.get("unexpected"))
                .and_then(|v| v.as_u64())
        })
        .unwrap_or(0);
    let skipped = stats
        .and_then(|s| s.get("skipped"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    (passed, failed, skipped, no_tests)
}

fn print_summary(score: &PlayhouseScore, engines: &[EngineResult]) {
    println!();
    println!("===========================================");
    println!("  Playhouse Stars: {} / 100  {}", score.stars, score.grade_emoji);
    println!("  Grade: {}", score.grade);
    println!("===========================================");
    println!();
    for cat in &score.categories {
        if cat.skipped {
            continue;
        }
        println!("  {:<28} {:>3}/100  {}", cat.label, cat.stars, cat.summary);
    }
    println!();
    println!("Why:");
    for line in &score.why {
        println!("  - {line}");
    }
    println!();
    println!("Engine exit codes:");
    for er in engines {
        if er.skipped {
            println!("  [-] {} (skipped)", er.engine);
        } else {
            let icon = if er.exit_code == 0 { "[*]" } else { "[x]" };
            println!("  {icon} {}: exit {}", er.engine, er.exit_code);
        }
    }
    println!();
    println!("Report: .playhouse/reports/score.json");
}

pub fn audit_json(report: &AuditReport) -> serde_json::Value {
    serde_json::json!({
        "command": "audit",
        "exitCode": report.exit_code,
        "playhouseScore": report.score,
        "engines": report.engines,
        "doctor": report.doctor,
    })
}
