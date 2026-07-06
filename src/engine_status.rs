//! Shared engine run classification for stdout, audit JSON, score reports, and TUI.

use crate::score::{EngineResult, is_implicit_penalty_skip};

/// How an engine participated in verify.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineRunKind {
    /// Engine ran and produced a result.
    Ran,
    /// User opt-out or N/A stack. Weight rebalances across remaining categories.
    ExplicitSkip,
    /// No URL or unreachable endpoint with `skip_lighthouse_without_server`. Scores 0/100, weight kept.
    ImplicitPenalty,
}

impl EngineRunKind {
    pub fn from_engine(er: &EngineResult) -> Self {
        if !er.skipped {
            return Self::Ran;
        }
        if is_implicit_penalty_skip(er) {
            return Self::ImplicitPenalty;
        }
        Self::ExplicitSkip
    }
}

/// Human-readable engine line for summaries and TUI.
pub fn engine_label(er: &EngineResult) -> String {
    match EngineRunKind::from_engine(er) {
        EngineRunKind::Ran => format!("{}: exit {}", er.engine, er.exit_code),
        EngineRunKind::ExplicitSkip => format!("{} (skipped)", er.engine),
        EngineRunKind::ImplicitPenalty => {
            let reason = er
                .metrics
                .get("reason")
                .and_then(|r| r.as_str())
                .unwrap_or("browser audit not run");
            format!("{} (not run: {})", er.engine, short_reason(reason))
        }
    }
}

/// Progress detail when a browser audit did not run.
pub fn browser_not_run_detail(reason: &str) -> String {
    if reason.starts_with("url-unreachable") {
        format!("URL not reachable; scored 0/100 ({})", short_reason(reason))
    } else {
        format!("No URL; scored 0/100 ({})", short_reason(reason))
    }
}

/// Short reason token for display (strips known prefixes).
pub fn short_reason(reason: &str) -> &str {
    reason
        .strip_prefix("no-url: ")
        .or_else(|| reason.strip_prefix("url-unreachable: "))
        .unwrap_or(reason)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::{implicit_penalty, skipped, EngineResult};

    fn ran(exit: i32) -> EngineResult {
        EngineResult {
            engine: "trivy".into(),
            exit_code: exit,
            skipped: false,
            metrics: serde_json::json!({}),
        }
    }

    #[test]
    fn explicit_skip_label() {
        let er = skipped("functional");
        assert_eq!(engine_label(&er), "functional (skipped)");
        assert_eq!(EngineRunKind::from_engine(&er), EngineRunKind::ExplicitSkip);
    }

    #[test]
    fn implicit_penalty_not_labeled_skipped() {
        let er = implicit_penalty("arkenar", "no-url: set default_url");
        let label = engine_label(&er);
        assert!(!label.contains("(skipped)"));
        assert!(label.contains("not run"));
        assert_eq!(
            EngineRunKind::from_engine(&er),
            EngineRunKind::ImplicitPenalty
        );
    }

    #[test]
    fn ran_engine_shows_exit_code() {
        let er = ran(4);
        assert_eq!(engine_label(&er), "trivy: exit 4");
    }
}
