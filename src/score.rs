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
            if is_implicit_penalty_skip(er) {
                if let Some(cat) = score_implicit_penalty(er) {
                    categories.push(cat);
                }
            }
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
    let all_engines_ok = engines.iter().all(engine_passes_gate);
    let meets_threshold = stars >= settings.star_pass_threshold;
    let passed = meets_threshold && all_engines_ok;

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
Each engine normalizes to 0-100, then categories are weighted (Trivy 25%, Functional 25%, \
Arkenar 20%, Lighthouse 20%, Toolchain 10%). Explicit skips (settings, N/A stack) rebalance weights. \
Missing or unreachable browser audits score 0/100 without rebalancing. \
90+ Production Ready, 75+ Good, 60+ Fair, 40+ Needs Work, below 40 Critical.";

pub fn is_implicit_penalty_skip(er: &EngineResult) -> bool {
    er.skipped
        && er
            .metrics
            .get("implicitPenalty")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
}

fn engine_passes_gate(er: &EngineResult) -> bool {
    let is_skipped = er.skipped;
    let ran_clean = engine_ok(er);
    is_skipped || ran_clean
}

fn engine_ok(er: &EngineResult) -> bool {
    if er.exit_code != 0 {
        return false;
    }
    if er.metrics.get("error").is_some() {
        return false;
    }
    if metrics_flag(er, "parseError") {
        return false;
    }
    if metrics_flag(er, "reportParseError") {
        return false;
    }
    if scan_incomplete(er) {
        return false;
    }
    true
}

fn metrics_flag(er: &EngineResult, key: &str) -> bool {
    er.metrics
        .get(key)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn scan_incomplete(er: &EngineResult) -> bool {
    er.metrics
        .get("scanComplete")
        .and_then(|v| v.as_bool())
        == Some(false)
}

fn engine_failed_scan(er: &EngineResult) -> bool {
    metrics_flag(er, "reportParseError") || er.metrics.get("error").is_some() || scan_incomplete(er)
}

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
    if engine_failed_scan(er) {
        let summary = er
            .metrics
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("Trivy scan incomplete")
            .to_string();
        return CategoryScore {
            id: "security_static".into(),
            label: "Security (Trivy)".into(),
            stars: 0,
            weight: 0.25,
            summary,
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
    if engine_failed_scan(er) {
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
    let scan_failed = er.metrics.get("error").is_some() || scan_incomplete(er);
    if scan_failed {
        let summary = er
            .metrics
            .get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("Lighthouse scan incomplete")
            .to_string();
        return CategoryScore {
            id: "performance".into(),
            label: "Performance and UX (Lighthouse)".into(),
            stars: 0,
            weight: 0.20,
            summary,
            details: vec![],
            skipped: false,
        };
    }
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
        0
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
            "skipKind": "explicit",
            "reason": reason,
        }),
    }
}

/// Browser audit skipped (no URL or unreachable). Scores 0/100, weight kept.
pub fn implicit_penalty(engine: &str, reason: &str) -> EngineResult {
    EngineResult {
        engine: engine.into(),
        exit_code: 0,
        skipped: true,
        metrics: serde_json::json!({
            "skipped": true,
            "implicitPenalty": true,
            "skipKind": "implicit",
            "reason": reason,
        }),
    }
}

/// Browser audit required but did not run. Fails verify.
pub fn browser_required_failure(engine: &str, reason: &str) -> EngineResult {
    EngineResult {
        engine: engine.into(),
        exit_code: 1,
        skipped: false,
        metrics: serde_json::json!({
            "passed": false,
            "scanComplete": false,
            "error": reason,
            "exitCode": 1,
        }),
    }
}

fn score_implicit_penalty(er: &EngineResult) -> Option<CategoryScore> {
    let reason = er
        .metrics
        .get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("browser audit not run");
    let (id, label, weight) = match er.engine.as_str() {
        "arkenar" => ("security_dast", "Security (Arkenar DAST)", 0.20),
        "lighthouse" => ("performance", "Performance and UX (Lighthouse)", 0.20),
        _ => return None,
    };
    Some(CategoryScore {
        id: id.into(),
        label: label.into(),
        stars: 0,
        weight,
        summary: reason.into(),
        details: vec!["Browser audit did not run; scored 0/100".into()],
        skipped: false,
    })
}

pub fn skipped(engine: &str) -> EngineResult {
    skipped_reason(engine, "skipped")
}

pub fn load_saved_report(workspace: &str) -> Option<(PlayhouseScore, Vec<EngineResult>, i32)> {
    let path = crate::tools::playhouse_dir(workspace)
        .join("reports")
        .join("score.json");
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let score: PlayhouseScore = serde_json::from_value(v.get("playhouseScore")?.clone()).ok()?;
    let engines: Vec<EngineResult> = v
        .get("engines")
        .and_then(|e| serde_json::from_value(e.clone()).ok())
        .unwrap_or_default();
    let exit_code = if score.passed { 0 } else { 1 };
    Some((score, engines, exit_code))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PlayhouseSettings;
    use crate::types::{CheckStatus, HealthCheck};

    fn settings() -> PlayhouseSettings {
        PlayhouseSettings::default()
    }

    fn is_explicit_skip(er: &EngineResult) -> bool {
        er.skipped && !is_implicit_penalty_skip(er)
    }

    fn engine(name: &str, exit: i32, metrics: serde_json::Value) -> EngineResult {
        EngineResult {
            engine: name.into(),
            exit_code: exit,
            skipped: false,
            metrics,
        }
    }

    #[test]
    fn explicit_skip_vs_implicit_penalty() {
        let explicit = skipped("functional");
        assert!(is_explicit_skip(&explicit));
        assert!(!is_implicit_penalty_skip(&explicit));
        let implicit = implicit_penalty("arkenar", "no-url");
        assert!(is_implicit_penalty_skip(&implicit));
        assert!(!is_explicit_skip(&implicit));
    }

    #[test]
    fn explicit_skipped_engines_rebalance_weights() {
        let engines = vec![
            skipped("functional"),
            engine(
                "trivy",
                0,
                serde_json::json!({
                    "summary": { "vulnerabilities": 0, "secrets": 0 },
                    "passed": true
                }),
            ),
        ];
        let doctor = vec![HealthCheck {
            name: "Trivy".into(),
            status: CheckStatus::Pass,
            detail: "ok".into(),
        }];
        let score = compute(&engines, Some(&doctor), &settings());
        assert!(score.stars > 0);
        assert!(score.passed);
    }

    #[test]
    fn implicit_browser_skip_scores_zero_without_rebalance() {
        let engines = vec![
            engine(
                "trivy",
                0,
                serde_json::json!({
                    "summary": { "vulnerabilities": 0, "secrets": 0 },
                    "passed": true
                }),
            ),
            engine(
                "functional",
                0,
                serde_json::json!({
                    "runner": "playwright",
                    "stats": { "passed": 10, "failed": 0, "skipped": 0 }
                }),
            ),
            implicit_penalty("arkenar", "no-url"),
            implicit_penalty("lighthouse", "no-url"),
        ];
        let score = compute(&engines, None, &settings());
        assert!(score.stars < 75);
        assert!(!score.passed);
        let arkenar = score
            .categories
            .iter()
            .find(|c| c.id == "security_dast")
            .unwrap();
        assert_eq!(arkenar.stars, 0);
    }

    #[test]
    fn lighthouse_empty_scores_not_perfect() {
        let engines = vec![engine(
            "lighthouse",
            0,
            serde_json::json!({
                "scanComplete": true,
                "scores": {}
            }),
        )];
        let score = compute(&engines, None, &settings());
        let lh = score
            .categories
            .iter()
            .find(|c| c.id == "performance")
            .unwrap();
        assert_eq!(lh.stars, 0);
    }

    #[test]
    fn parse_error_fails_pass_gate() {
        let engines = vec![engine(
            "playwright",
            0,
            serde_json::json!({ "parseError": true }),
        )];
        let score = compute(&engines, None, &settings());
        assert!(!score.passed);
    }

    #[test]
    fn trivy_incomplete_scan_scores_zero() {
        let engines = vec![engine(
            "trivy",
            5,
            serde_json::json!({
                "scanComplete": false,
                "error": "trivy returned empty output"
            }),
        )];
        let score = compute(&engines, None, &settings());
        let trivy = score
            .categories
            .iter()
            .find(|c| c.id == "security_static")
            .unwrap();
        assert_eq!(trivy.stars, 0);
        assert!(!score.passed);
    }

    #[test]
    fn trivy_vulns_lower_stars() {
        let engines = vec![engine(
            "trivy",
            4,
            serde_json::json!({
                "summary": { "vulnerabilities": 2, "secrets": 0 },
                "passed": false
            }),
        )];
        let score = compute(&engines, None, &settings());
        assert!(score.stars < 100);
        assert!(!score.passed);
    }

    #[test]
    fn functional_all_pass_scores_high() {
        let engines = vec![engine(
            "functional",
            0,
            serde_json::json!({
                "runner": "cargo-test",
                "stats": { "passed": 10, "failed": 0, "skipped": 0 }
            }),
        )];
        let score = compute(&engines, None, &settings());
        let functional = score
            .categories
            .iter()
            .find(|c| c.id == "functional")
            .unwrap();
        assert_eq!(functional.stars, 100);
    }

    #[test]
    fn load_saved_report_roundtrip() {
        let dir = std::env::temp_dir().join(format!("playhouse-score-load-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".playhouse/reports")).unwrap();
        let ws = dir.to_str().unwrap();
        let engines = vec![skipped("functional")];
        let stars = compute(&engines, None, &settings());
        save_report(ws, &stars, &engines).unwrap();
        let loaded = load_saved_report(ws).expect("saved report");
        assert_eq!(loaded.0.stars, stars.stars);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
