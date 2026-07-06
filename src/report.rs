use crate::types::LighthouseScores;

pub fn format_lighthouse_text(url: &str, scores: &LighthouseScores) -> String {
    format!(
        "Lighthouse Audit: {url}\n\
          Performance:    {}\n\
          Accessibility:  {}\n\
          Best Practices: {}\n\
          SEO:            {}",
        LighthouseScores::score_label(scores.performance),
        LighthouseScores::score_label(scores.accessibility),
        LighthouseScores::score_label(scores.best_practices),
        LighthouseScores::score_label(scores.seo),
    )
}

/// Write per-engine JSON to `.playhouse/reports/{name}.json`.
pub fn save_engine_report(
    workspace: &str,
    name: &str,
    data: &serde_json::Value,
) -> std::io::Result<std::path::PathBuf> {
    let dir = crate::tools::playhouse_dir(workspace).join("reports");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{name}.json"));
    std::fs::write(&path, serde_json::to_string_pretty(data).unwrap_or_default())?;
    Ok(path)
}
