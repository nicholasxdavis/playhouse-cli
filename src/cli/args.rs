use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "playhouse",
    about = "QA CLI - Playwright, Lighthouse, Trivy, and tool health checks",
    version,
    arg_required_else_help = false
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output machine-readable JSON instead of human text
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check which tools are installed and ready
    Doctor,

    /// Install bundled tools (default: full web profile)
    Install {
        /// Trivy + Arkenar only (no Playwright/Lighthouse)
        #[arg(long, conflicts_with = "full")]
        minimal: bool,

        /// Playwright + Lighthouse + chromium (default when neither flag is set)
        #[arg(long, conflicts_with = "minimal")]
        full: bool,
    },

    /// Initialize .playhouse/ workspace, install tools, export brief
    Init {
        /// Enable stay-on-track skill (.playhouse/stay-on-track/SKILL.md)
        #[arg(long)]
        stay_on_track: bool,
    },

    /// Full agent manifest, status, plan, or handoff bundle
    Agent {
        #[command(subcommand)]
        action: Option<AgentAction>,
    },

    /// Show or change global and workspace configuration
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },

    /// Export .playhouse/BRIEF.md
    Export,

    /// Stay-on-track skill management
    StayOnTrack {
        #[command(subcommand)]
        action: StayOnTrackAction,
    },

    /// Playhouse agent skill (.playhouse/SKILL.md) - recommended for agents
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },

    /// Run Lighthouse audit against a URL
    Lighthouse {
        /// Target URL (auto-detects local dev server if omitted)
        url: Option<String>,
    },

    /// Run Playwright tests in the workspace
    Playwright {
        /// Optional test file or grep pattern
        pattern: Option<String>,
    },

    /// Run detected functional test runner (playwright, cargo, go, pytest, npm test, …)
    Functional,

    /// Scaffold and run test baseplates
    Test {
        #[command(subcommand)]
        action: TestAction,
    },

    /// Run Trivy filesystem security scan
    Trivy,

    /// Run Arkenar DAST web scan (MIT Rust - replaces OWASP ZAP)
    Arkenar {
        /// Target URL (auto-detects local dev server if omitted)
        url: Option<String>,
    },

    /// Run all verification suites
    Verify {
        /// Target URL for browser-based checks
        #[arg(long)]
        url: Option<String>,
    },

    /// Show or compute Playhouse Star Rating (0–100 audit score)
    Score {
        /// Target URL for Lighthouse + Arkenar (auto-detects local server)
        #[arg(long)]
        url: Option<String>,

        /// Show last saved score from .playhouse/reports/score.json
        #[arg(long)]
        last: bool,
    },

    /// Check for newer releases on GitHub and npm
    Upgrade,
}

#[derive(Subcommand)]
pub enum AgentAction {
    /// Quick health, last score, and recommended next actions
    Status,
    /// Phased workflow plan for this workspace
    Plan,
    /// Run verify and write .playhouse/AGENT.json handoff bundle
    Handoff {
        #[arg(long)]
        url: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// List settable config keys and types
    Schema,
    /// Read a setting by key
    Get { key: String },
    /// Update a setting by key
    Set { key: String, value: String },
}

#[derive(Subcommand)]
pub enum StayOnTrackAction {
    /// Enable stay-on-track and spawn .playhouse/stay-on-track/SKILL.md
    Enable,
    /// Disable stay-on-track flag
    Disable,
    /// Show stay-on-track status
    Status,
}

#[derive(Subcommand)]
pub enum TestAction {
    /// List available baseplates for this stack
    List,

    /// Scaffold the default or specified baseplate
    Init {
        /// Baseplate id (default: stack-appropriate plate from `playhouse test list`)
        #[arg(long)]
        plate: Option<String>,

        /// Overwrite when tests already exist
        #[arg(long)]
        force: bool,
    },

    /// Add an additional baseplate (does not block on existing tests)
    Add {
        #[arg(long)]
        plate: String,

        /// Overwrite if the target file already exists
        #[arg(long)]
        force: bool,
    },

    /// Run functional tests via the detected runner
    Run,
}

#[derive(Subcommand)]
pub enum SkillAction {
    /// Install or refresh .playhouse/SKILL.md
    Install,
    /// Enable playhouse skill for this workspace
    Enable,
    /// Disable playhouse skill flag
    Disable,
    /// Show playhouse skill status
    Status,
}
