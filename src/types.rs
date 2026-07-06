use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: CheckStatus,
    pub detail: String,
}

impl HealthCheck {
    pub fn pass(name: &str, detail: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Pass,
            detail: detail.to_string(),
        }
    }

    pub fn warn(name: &str, detail: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Warn,
            detail: detail.to_string(),
        }
    }

    pub fn fail(name: &str, detail: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Fail,
            detail: detail.to_string(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self.status {
            CheckStatus::Pass => "[*]",
            CheckStatus::Warn => "[!]",
            CheckStatus::Fail => "[x]",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LighthouseScores {
    pub performance: Option<f64>,
    pub accessibility: Option<f64>,
    pub best_practices: Option<f64>,
    pub seo: Option<f64>,
}

impl LighthouseScores {
    pub fn score_label(score: Option<f64>) -> String {
        match score {
            Some(s) => format!("{:.0}", s * 100.0),
            None => "N/A".to_string(),
        }
    }

    pub fn all_pass(&self, threshold: f64) -> bool {
        [self.performance, self.accessibility, self.best_practices, self.seo]
            .iter()
            .all(|s| s.is_some_and(|v| v >= threshold))
    }
}
