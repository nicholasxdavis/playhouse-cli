use serde::{Deserialize, Serialize};

use crate::config::PlayhouseSettings;
use crate::types::{CheckStatus, HealthCheck, LighthouseScores};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineResult {
    pub engine: String,
    pub exit_code: i32,
    pub skipped: bool,
    pub metrics: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    pub id: String,
    pub label: String,
    pub stars: u8,
    pub weight: f64,
    pub summary: String,
    pub details: Vec<String>,
    pub skipped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayhouseScore {
    pub stars: u8,
    pub grade: String,
    pub grade_emoji: String,
    pub passed: bool,
    pub categories: Vec<CategoryScore>,
    pub why: Vec<String>,
    pub methodology: String,
}

pub fn compute(
    engines: &[EngineResult],
    doctor: Option<&[HealthCheck]>,
    settings: &PlayhouseSettings,
) -> PlayhouseScore {
    let mut categories = Vec::new();

    if let Some(checks) = doctor {
        let pass = checks.iter().filter(|c| c.status == CheckStatus::Pass).count();
        let total = checks.len().max(1);
        let pct = (pass as f64 / total as f64 * 100.0).round() as u8;
        let mut details: Vec<String> = checks
            .iter()
            .filter(|c| c.status != CheckStatus::Pass)
            .map(|c| format!("{}: {}", c.name, c.detail))
            .collect();
        if details.is_empty() {
            details.push(format!("{pass}/{total} tools ready"));
        }
        categories.push(CategoryScore {
            id: "tools".into(),
            label: "Toolchain".into(),
            stars: pct,
            weight: 0.10,
            summary: format!("{pass}/{total} tools ready"),
            details,
            skipped: false,
        });
    }

    for er in engines {
        if er.skipped {
            continue;
        }
        match er.engine.as_str() {
            "trivy" => categories.push(score_trivy(er)),
            "functional" | "playwright" => categories.push(score_functional(er)),
            "arkenar" => categories.push(score_arkenar(er)),
            "lighthouse" => categories.push(score_lighthouse(er, settings)),
            _ => {}
        }
    }

    let stars = weighted_stars(&categories);
    let grade = grade_for(stars);
    let why = build_why(&categories, stars);
    let passed = stars >= settings.star_pass_threshold
        && engines.iter().all(|e| e.skipped || e.exit_code == 0);

    PlayhouseScore {
        stars,
        grade: grade.0.to_string(),
        grade_emoji: grade.1.to_string(),
        passed,
        categories,
        why,
        methodology: METHODOLOGY.to_string(),
    }
}

pub const METHODOLOGY: &str = "Playhouse Stars (0-100) combine weighted category scores inspired by Lighthouse. \
Each engine normalizes to 0-100, then categories are weighted (Security 45%, Functional 25%, \
Performance and UX 20%, Toolchain 10%). Skipped engines are excluded and weights rebalance. \
90+ Production Ready, 75+ Good, 60+ Fair, 40+ Needs Work, below 40 Critical.";

fn weighted_stars(categories: &[CategoryScore]) -> u8 {
    let active: Vec<_> = categories.iter().filter(|c| !c.skipped).collect();
    if active.is_empty() {
        return 0;
    }
    let total_weight: f64 = active.iter().map(|c| c.weight).sum();
    if total_weight <= 0.0 {
        return 0;
    }
    let sum: f64 = active
        .iter()
        .map(|c| c.stars as f64 * (c.weight / total_weight))
        .sum();
    sum.round().clamp(0.0, 100.0) as u8
}

fn grade_for(stars: u8) -> (&'static str, &'static str) {
    match stars {
        90..=100 => ("Production Ready", "*****"),
        75..=89 => ("Good", "**** "),
        60..=74 => ("Fair", "***  "),
        40..=59 => ("Needs Work", "**   "),
        _ => ("Critical", "*    "),
    }
}

fn build_why(categories: &[CategoryScore], stars: u8) -> Vec<String> {
    let mut why = Vec::new();
    why.push(format!(
        "Overall: {stars}/100 ({})",
        grade_for(stars).0
    ));

    for cat in categories {
        if cat.skipped {
            continue;
        }
        let level = if cat.stars >= 75 {
            "Strong"
        } else if cat.stars >= 50 {
            "Moderate"
        } else {
            "Weak"
        };
        why.push(format!(
            "{level} {}: {}/100 ({})",
            cat.label, cat.stars, cat.summary
        ));
    }

    let weak: Vec<_> = categories
        .iter()
        .filter(|c| !c.skipped && c.stars < 60)
        .map(|c| c.label.as_str())
        .collect();
    if !weak.is_empty() {
        why.push(format!(
            "Improve: {}. Re-run `playhouse verify --json` after fixes.",
            weak.join(", ")
        ));
    }

    why
}

fn score_trivy(er: &EngineResult) -> CategoryScore {
    if let Some(err) = er.metrics.get("error").and_then(|e| e.as_str()) {
        return CategoryScore {
            id: "security_static".into(),
            label: "Security (Trivy)".into(),
            stars: 0,
            weight: 0.25,
            summary: err.into(),
            details: vec![],
            skipped: false,
        };
    }
    let vulns = er.metrics["summary"]["vulnerabilities"].as_u64().unwrap_or(0);
    let secrets = er.metrics["summary"]["secrets"].as_u64().unwrap_or(0);
    let penalty = (vulns * 15 + secrets * 25).min(100);
    let stars = (100_i64.saturating_sub(penalty as i64)).clamp(0, 100) as u8;
    let passed = er.metrics["passed"].as_bool().unwrap_or(er.exit_code == 0);
    CategoryScore {
        id: "security_static".into(),
        label: "Security (Trivy)".into(),
        stars,
        weight: 0.25,
        summary: if passed {
            "No high/critical vulns or secrets".into()
        } else {
            format!("{vulns} vulns, {secrets} secrets")
        },
        details: vec![
            format!("Vulnerabilities (HIGH/CRITICAL): {vulns}"),
            format!("Secrets detected: {secrets}"),
        ],
        skipped: false,
    }
}

fn score_functional(er: &EngineResult) -> CategoryScore {
    let runner = er
        .metrics
        .get("runner")
        .and_then(|v| v.as_str())
        .unwrap_or("functional");
    let label = format!("Functional ({runner})");

    if er.metrics.get("noTests").and_then(|v| v.as_bool()).unwrap_or(false) {
        return CategoryScore {
            id: "functional".into(),
            label,
            stars: 0,
            weight: 0.25,
            summary: "No tests ran".into(),
            details: vec!["Add tests or run `playhouse test init`".into()],
            skipped: false,
        };
    }
    if let Some(err) = er.metrics.get("error").and_then(|e| e.as_str()) {
        return CategoryScore {
            id: "functional".into(),
            label,
            stars: 0,
            weight: 0.25,
            summary: err.into(),
            details: vec![],
            skipped: false,
        };
    }
    let stats = &er.metrics["stats"];
    let passed = stats["passed"].as_u64().unwrap_or_else(|| {
        stats["expected"].as_u64().unwrap_or(0)
    });
    let failed = stats["failed"].as_u64().unwrap_or_else(|| {
        stats["unexpected"].as_u64().unwrap_or(0)
    });
    let skipped = stats["skipped"].as_u64().unwrap_or(0);
    let stars = if passed == 0 && failed == 0 {
        0
    } else {
        let total = passed + failed;
        ((passed as f64 / total as f64) * 100.0).round() as u8
    };
    CategoryScore {
        id: "functional".into(),
        label,
        stars,
        weight: 0.25,
        summary: format!("{passed} passed, {failed} failed, {skipped} skipped"),
        details: vec![
            format!("Passed: {passed}"),
            format!("Failed: {failed}"),
        ],
        skipped: false,
    }
}

fn score_arkenar(er: &EngineResult) -> CategoryScore {
    if er.metrics.get("reportParseError").and_then(|v| v.as_bool()).unwrap_or(false)
        || er.metrics.get("error").is_some()
    {
        let summary = er
            .metrics
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("Arkenar report not parsed")
            .to_string();
        return CategoryScore {
            id: "security_dast".into(),
            label: "Security (Arkenar DAST)".into(),
            stars: 0,
            weight: 0.20,
            summary,
            details: vec![],
            skipped: false,
        };
    }
    let high = er.metrics["summary"]["high"].as_u64().unwrap_or(0);
    let medium = er.metrics["summary"]["medium"].as_u64().unwrap_or(0);
    let low = er.metrics["summary"]["low"].as_u64().unwrap_or(0);
    let penalty = (high * 25 + medium * 10 + low * 2).min(100);
    let stars = (100_i64.saturating_sub(penalty as i64)).clamp(0, 100) as u8;
    CategoryScore {
        id: "security_dast".into(),
        label: "Security (Arkenar DAST)".into(),
        stars,
        weight: 0.20,
        summary: format!("high={high} medium={medium} low={low}"),
        details: vec![
            format!("High: {high}"),
            format!("Medium: {medium}"),
            format!("Low: {low}"),
        ],
        skipped: false,
    }
}

fn score_lighthouse(er: &EngineResult, settings: &PlayhouseSettings) -> CategoryScore {
    let scores_json = &er.metrics["scores"];
    let lh = LighthouseScores {
        performance: scores_json["performance"].as_f64(),
        accessibility: scores_json["accessibility"].as_f64(),
        best_practices: scores_json["bestPractices"].as_f64(),
        seo: scores_json["seo"].as_f64(),
    };
    let values: Vec<f64> = [lh.performance, lh.accessibility, lh.best_practices, lh.seo]
        .into_iter()
        .flatten()
        .collect();
    let stars = if values.is_empty() {
        if er.exit_code == 0 { 100 } else { 0 }
    } else {
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        (avg * 100.0).round() as u8
    };
    let threshold_pct = (settings.lighthouse_threshold * 100.0).round() as u8;
    CategoryScore {
        id: "performance".into(),
        label: "Performance and UX (Lighthouse)".into(),
        stars,
        weight: 0.20,
        summary: format!(
            "perf {} a11y {} bp {} seo {} (min {threshold_pct}%)",
            LighthouseScores::score_label(lh.performance),
            LighthouseScores::score_label(lh.accessibility),
            LighthouseScores::score_label(lh.best_practices),
            LighthouseScores::score_label(lh.seo),
        ),
        details: vec![
            format!("Performance: {}", LighthouseScores::score_label(lh.performance)),
            format!("Accessibility: {}", LighthouseScores::score_label(lh.accessibility)),
            format!("Best Practices: {}", LighthouseScores::score_label(lh.best_practices)),
            format!("SEO: {}", LighthouseScores::score_label(lh.seo)),
        ],
        skipped: false,
    }
}

pub fn skipped_reason(engine: &str, reason: &str) -> EngineResult {
    EngineResult {
        engine: engine.into(),
        exit_code: 0,
        skipped: true,
        metrics: serde_json::json!({
            "skipped": true,
            "reason": reason,
        }),
    }
}

pub fn skipped(engine: &str) -> EngineResult {
    skipped_reason(engine, "skipped")
}

pub fn save_report(
    workspace: &str,
    score: &PlayhouseScore,
    engines: &[EngineResult],
) -> std::io::Result<()> {
    let dir = crate::tools::playhouse_dir(workspace).join("reports");
    std::fs::create_dir_all(&dir)?;
    let report = serde_json::json!({
        "playhouseScore": score,
        "engines": engines,
        "generatedAt": timestamp(),
    });
    std::fs::write(
        dir.join("score.json"),
        serde_json::to_string_pretty(&report).unwrap_or_default(),
    )
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}
