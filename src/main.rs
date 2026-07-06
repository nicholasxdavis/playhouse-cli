mod agent;
mod audit;
mod baseplates;
mod cmd;
mod config;
mod config_cli;
mod detect;
mod engines;
mod install;
mod pkgmgr;
mod project;
mod report;
mod score;
mod tools;
mod tui;
mod types;
mod upgrade;
mod workspace;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "playhouse",
    about = "QA CLI - Playwright, Lighthouse, Trivy, and tool health checks",
    version,
    arg_required_else_help = false
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Output machine-readable JSON instead of human text
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
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
enum AgentAction {
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
enum ConfigAction {
    /// List settable config keys and types
    Schema,
    /// Read a setting by key
    Get {
        key: String,
    },
    /// Update a setting by key
    Set {
        key: String,
        value: String,
    },
}

#[derive(Subcommand)]
enum StayOnTrackAction {
    /// Enable stay-on-track and spawn .playhouse/stay-on-track/SKILL.md
    Enable,
    /// Disable stay-on-track flag
    Disable,
    /// Show stay-on-track status
    Status,
}

#[derive(Subcommand)]
enum TestAction {
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
enum SkillAction {
    /// Install or refresh .playhouse/SKILL.md
    Install,
    /// Enable playhouse skill for this workspace
    Enable,
    /// Disable playhouse skill flag
    Disable,
    /// Show playhouse skill status
    Status,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let workspace = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .to_string_lossy()
        .to_string();

    let settings = config::load_settings();
    workspace::maybe_auto_init(&workspace, &settings);

    let json = cli.json || settings.json_output_default || settings.agent_mode;

    let exit_code = match cli.command {
        None => tui::run(&workspace).await,

        Some(Commands::Doctor) => {
            if settings.auto_install_tools {
                let profile = project::detect(&workspace);
                let _ = install::ensure_profile(
                    &workspace,
                    profile.install_profile(),
                    json,
                )
                .await;
            }
            let checks = detect::run_doctor(&workspace);
            if json {
                println!("{}", serde_json::to_string_pretty(&checks).unwrap_or_default());
            } else {
                for check in &checks {
                    println!("{} {} -- {}", check.icon(), check.name, check.detail);
                }
            }
            if checks.iter().any(|c| matches!(c.status, types::CheckStatus::Fail)) {
                1
            } else {
                0
            }
        }

        Some(Commands::Install { minimal, full }) => {
            let profile = install::InstallProfile::from_flags(minimal, full);
            let report = install::ensure_profile(&workspace, profile, json).await;
            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            } else {
                for msg in &report.messages {
                    println!("[*] {msg}");
                }
                for err in &report.errors {
                    eprintln!("[x] {err}");
                }
            }
            if report.errors.is_empty() { 0 } else { 5 }
        }

        Some(Commands::Init { stay_on_track }) => {
            let enable_sot = stay_on_track || settings.stay_on_track_enabled;
            match workspace::init_workspace(
                &workspace,
                &settings,
                true,
                enable_sot,
                json,
            )
            .await
            {
                Ok(report) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
                    } else {
                        println!("[*] Initialized {}", report.playhouse_dir);
                        for msg in &report.tools.messages {
                            println!("[*] {msg}");
                        }
                        if report.stay_on_track {
                            if let Some(skill) = &report.skill_path {
                                println!("[*] Stay-on-track skill: {skill}");
                            }
                        }
                        if report.playhouse_skill {
                            if let Some(skill) = &report.playhouse_skill_path {
                                println!("[*] Playhouse agent skill: {skill}");
                            }
                        }
                        println!("[*] Brief: {}", report.brief_path);
                    }
                    0
                }
                Err(e) => {
                    if json {
                        let out = serde_json::json!({ "error": e });
                        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                    } else {
                        eprintln!("[x] Init failed: {e}");
                    }
                    1
                }
            }
        }

        Some(Commands::Agent { action }) => match action {
            None => {
                let manifest = agent::manifest(&workspace);
                println!("{}", serde_json::to_string_pretty(&manifest).unwrap_or_default());
                0
            }
            Some(AgentAction::Status) => {
                let out = agent::status(&workspace);
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                0
            }
            Some(AgentAction::Plan) => {
                let out = agent::plan(&workspace);
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                0
            }
            Some(AgentAction::Handoff { url }) => {
                run_agent_handoff(&workspace, url.as_deref(), json, &settings).await
            }
        }

        Some(Commands::Config { action }) => match action {
            None => {
                let ws = workspace::load_workspace_config(&workspace);
                let out = serde_json::json!({
                    "global": settings,
                    "workspace": ws,
                    "paths": {
                        "playhouseHome": config::playhouse_home(),
                        "settingsFile": config::settings_path(),
                        "workspaceConfig": workspace::workspace_config_path(&workspace),
                        "bundledTrivy": tools::bundled_trivy_path(),
                        "playwrightPrefix": tools::playwright_prefix(&workspace),
                        "agentHandoff": workspace::agent_json_path(&workspace),
                    },
                    "schema": "playhouse config schema --json",
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                0
            }
            Some(ConfigAction::Schema) => {
                println!("{}", serde_json::to_string_pretty(&config_cli::schema_json()).unwrap_or_default());
                0
            }
            Some(ConfigAction::Get { key }) => match config_cli::get(&workspace, &key) {
                Ok(v) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "key": key, "value": v })).unwrap_or_default());
                    } else {
                        println!("{key} = {}", serde_json::to_string(&v).unwrap_or_default());
                    }
                    0
                }
                Err(e) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "error": e })).unwrap_or_default());
                    } else {
                        eprintln!("[x] {e}");
                    }
                    1
                }
            },
            Some(ConfigAction::Set { key, value }) => match config_cli::set(&workspace, &key, &value) {
                Ok(v) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "key": key, "value": v, "saved": true })).unwrap_or_default());
                    } else {
                        println!("[*] Set {key} = {}", serde_json::to_string(&v).unwrap_or_default());
                    }
                    0
                }
                Err(e) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "error": e })).unwrap_or_default());
                    } else {
                        eprintln!("[x] {e}");
                    }
                    1
                }
            },
        }

        Some(Commands::Export) => {
            let ws = workspace::load_workspace_config(&workspace);
            let brief = agent::build_brief_text(&workspace, &settings, &ws);
            let path = tools::playhouse_dir(&workspace).join("BRIEF.md");
            let _ = std::fs::create_dir_all(path.parent().unwrap());
            match std::fs::write(&path, &brief) {
                Ok(()) => {
                    if json {
                        let out = serde_json::json!({ "path": path, "written": true });
                        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                    } else {
                        println!("[*] Exported {}", path.display());
                    }
                    0
                }
                Err(e) => {
                    if json {
                        let out = serde_json::json!({ "error": e.to_string() });
                        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                    } else {
                        eprintln!("[x] Export failed: {e}");
                    }
                    1
                }
            }
        }

        Some(Commands::StayOnTrack { action }) => match action {
            StayOnTrackAction::Enable => {
                match workspace::enable_stay_on_track_mode(&workspace, &settings) {
                    Ok(path) => {
                        if json {
                            let out = serde_json::json!({
                                "enabled": true,
                                "skillPath": path,
                            });
                            println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                        } else {
                            println!("[*] Stay-on-track enabled: {}", path.display());
                        }
                        0
                    }
                    Err(e) => {
                        if json {
                            let out = serde_json::json!({ "error": e });
                            println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                        } else {
                            eprintln!("[x] {e}");
                        }
                        1
                    }
                }
            }
            StayOnTrackAction::Disable => {
                match workspace::disable_stay_on_track_mode(&workspace) {
                    Ok(()) => {
                        if json {
                            println!(r#"{{"enabled":false}}"#);
                        } else {
                            println!("[*] Stay-on-track disabled for workspace");
                        }
                        0
                    }
                    Err(e) => {
                        eprintln!("[x] {e}");
                        1
                    }
                }
            }
            StayOnTrackAction::Status => {
                let status = workspace::stay_on_track_status(&workspace, &settings);
                println!("{}", serde_json::to_string_pretty(&status).unwrap_or_default());
                0
            }
        },

        Some(Commands::Skill { action }) => match action {
            SkillAction::Install | SkillAction::Enable => {
                match workspace::install_playhouse_skill(&workspace, &settings) {
                    Ok(path) => {
                        if json {
                            let out = serde_json::json!({
                                "enabled": true,
                                "recommended": true,
                                "skillPath": path,
                            });
                            println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                        } else {
                            println!("[*] Playhouse agent skill: {}", path.display());
                        }
                        0
                    }
                    Err(e) => {
                        if json {
                            let out = serde_json::json!({ "error": e });
                            println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                        } else {
                            eprintln!("[x] {e}");
                        }
                        1
                    }
                }
            }
            SkillAction::Disable => {
                match workspace::disable_playhouse_skill_mode(&workspace) {
                    Ok(()) => {
                        if json {
                            println!(r#"{{"enabled":false}}"#);
                        } else {
                            println!("[*] Playhouse agent skill disabled for workspace");
                        }
                        0
                    }
                    Err(e) => {
                        eprintln!("[x] {e}");
                        1
                    }
                }
            }
            SkillAction::Status => {
                let status = workspace::playhouse_skill_status(&workspace, &settings);
                println!("{}", serde_json::to_string_pretty(&status).unwrap_or_default());
                0
            }
        },

        Some(Commands::Lighthouse { url }) => {
            let target = resolve_url(&workspace, url, &settings);
            engines::lighthouse::run(&target, &workspace, json, false).await
        }

        Some(Commands::Playwright { pattern }) => {
            engines::playwright::run(&workspace, pattern.as_deref(), json, false).await
        }

        Some(Commands::Functional) => engines::functional::run(&workspace, json, false).await,

        Some(Commands::Test { action }) => match action {
            TestAction::List => {
                let profile = project::detect(&workspace);
                let plates = baseplates::list_plates(&profile);
                if json {
                    println!("{}", serde_json::to_string_pretty(&plates).unwrap_or_default());
                } else {
                    for p in &plates {
                        let mark = if p.compatible { "[*]" } else { "[ ]" };
                        println!(
                            "{mark} {} ({}) — {} [{}]",
                            p.id, p.label, p.description, p.runner
                        );
                    }
                }
                0
            }
            TestAction::Init { plate, force } => match baseplates::init_plate(
                &workspace,
                plate.as_deref(),
                force,
            ) {
                Ok(report) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
                    } else {
                        println!("[*] Applied baseplate: {}", report.plate);
                        for f in &report.files {
                            println!("[*] Wrote {f}");
                        }
                        for s in &report.skipped {
                            println!("[!] Skipped (exists): {s}");
                        }
                        if let Some(cfg) = &report.playwright_config {
                            println!("[*] Playwright config: {cfg}");
                        }
                        println!("[*] Manifest: {}", report.manifest_path);
                    }
                    0
                }
                Err(e) => {
                    if json {
                        let out = serde_json::json!({ "error": e });
                        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                    } else {
                        eprintln!("[x] {e}");
                    }
                    1
                }
            },
            TestAction::Add { plate, force } => match baseplates::add_plate(&workspace, &plate, force)
            {
                Ok(report) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
                    } else {
                        println!("[*] Added baseplate: {}", report.plate);
                        for f in &report.files {
                            println!("[*] Wrote {f}");
                        }
                        for s in &report.skipped {
                            println!("[!] Skipped (exists): {s}");
                        }
                        if let Some(cfg) = &report.playwright_config {
                            println!("[*] Playwright config: {cfg}");
                        }
                    }
                    0
                }
                Err(e) => {
                    if json {
                        let out = serde_json::json!({ "error": e });
                        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                    } else {
                        eprintln!("[x] {e}");
                    }
                    1
                }
            },
            TestAction::Run => engines::functional::run(&workspace, json, false).await,
        },

        Some(Commands::Trivy) => engines::trivy::run(&workspace, json, false).await,

        Some(Commands::Arkenar { url }) => {
            let target = resolve_url(&workspace, url, &settings);
            engines::arkenar::run(&target, &workspace, json, false).await
        }

        Some(Commands::Verify { url }) => {
            let target = url
                .or_else(|| workspace::resolve_verify_url(&workspace, &settings));
            run_verify(&workspace, target.as_deref(), json, &settings).await
        }

        Some(Commands::Score { url, last }) => {
            if last {
                let path = tools::playhouse_dir(&workspace).join("reports").join("score.json");
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        if json {
                            println!("{}", content);
                        } else {
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(score) = v.get("playhouseScore") {
                                    let stars = score["stars"].as_u64().unwrap_or(0);
                                    let grade = score["grade"].as_str().unwrap_or("?");
                                    println!("Playhouse Star Rating: {stars}/100 - {grade}");
                                    println!("Report: {}", path.display());
                                } else {
                                    println!("{}", content);
                                }
                            } else {
                                println!("{}", content);
                            }
                        }
                        0
                    }
                    Err(_) => {
                        if json {
                            let out = serde_json::json!({ "error": "no score report - run playhouse score or verify first" });
                            println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
                        } else {
                            eprintln!("[x] No score report - run `playhouse score` or `playhouse verify` first");
                        }
                        1
                    }
                }
            } else {
                let target = url.or_else(|| workspace::resolve_verify_url(&workspace, &settings));
                let report = audit::run_audit(&workspace, target.as_deref(), &settings, json).await;
                if json {
                    println!("{}", serde_json::to_string_pretty(&audit::audit_json(&report)).unwrap_or_default());
                }
                report.exit_code
            }
        }

        Some(Commands::Upgrade) => {
            let report = upgrade::check();
            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            } else {
                println!("Playhouse {}", report.current);
                println!("Install method: {}", report.install_method);
                if let Some(ref latest) = report.github.latest {
                    let mark = if report.github.update_available { "[*]" } else { "[ ]" };
                    println!("{mark} GitHub latest: v{latest} — {}", report.github.url);
                } else if let Some(ref err) = report.github.error {
                    println!("[!] GitHub check failed: {err}");
                }
                if let Some(ref latest) = report.npm.latest {
                    let mark = if report.npm.update_available { "[*]" } else { "[ ]" };
                    println!("{mark} npm latest: v{latest} — {}", report.npm.url);
                } else if let Some(ref err) = report.npm.error {
                    println!("[!] npm check failed: {err}");
                }
                if report.github.update_available || report.npm.update_available {
                    println!();
                    println!("Upgrade:");
                    println!("  {}", report.upgrade.npm);
                    println!("  {}", report.upgrade.cargo);
                    println!("  Releases: {}", report.upgrade.releases);
                } else {
                    println!();
                    println!("[*] You are on the latest published version (or offline).");
                }
            }
            0
        }
    };

    std::process::exit(exit_code);
}

pub(crate) async fn run_verify(
    workspace: &str,
    url: Option<&str>,
    json: bool,
    settings: &config::PlayhouseSettings,
) -> i32 {
    let resolved = url
        .map(String::from)
        .or_else(|| workspace::resolve_verify_url(workspace, settings));

    if resolved.is_none() && !json && !settings.skip_lighthouse_without_server {
        let hints = detect::port_hints(workspace);
        if hints.is_empty() {
            eprintln!("[!] No URL — browser audits skipped. Set: playhouse config set default_url http://localhost:PORT");
        } else {
            eprintln!(
                "[!] No URL — browser audits skipped. Start dev server or: playhouse config set default_url http://localhost:{}",
                hints[0]
            );
        }
    }

    let report = audit::run_audit(workspace, resolved.as_deref(), settings, json).await;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&audit::audit_json(&report)).unwrap_or_default()
        );
    }

    if settings.auto_export_agent_brief {
        let ws = workspace::load_workspace_config(workspace);
        let brief = agent::build_brief_text(workspace, settings, &ws);
        let path = tools::playhouse_dir(workspace).join("BRIEF.md");
        let _ = std::fs::write(&path, brief);
    }

    if settings.auto_export_handoff_json {
        let _ = agent::save_handoff_json(workspace, settings, Some(&report));
    }

    report.exit_code
}

async fn run_agent_handoff(
    workspace: &str,
    url: Option<&str>,
    json: bool,
    settings: &config::PlayhouseSettings,
) -> i32 {
    let target = url
        .map(|s| s.to_string())
        .or_else(|| workspace::resolve_verify_url(workspace, settings));

    if settings.auto_install_tools {
        let _ = install::ensure_all(workspace, true).await;
    }

    let report = audit::run_audit(workspace, target.as_deref(), settings, true).await;

    let ws = workspace::load_workspace_config(workspace);
    let brief = agent::build_brief_text(workspace, settings, &ws);
    let brief_path = tools::playhouse_dir(workspace).join("BRIEF.md");
    let _ = std::fs::write(&brief_path, brief);

    let agent_path = agent::save_handoff_json(workspace, settings, Some(&report))
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| ".playhouse/AGENT.json".into());

    if json {
        let out = serde_json::json!({
            "command": "agent handoff",
            "exitCode": report.exit_code,
            "passed": report.exit_code == 0,
            "playhouseScore": report.score,
            "paths": {
                "brief": brief_path,
                "agent": agent_path,
                "score": tools::playhouse_dir(workspace).join("reports/score.json"),
            },
            "audit": audit::audit_json(&report),
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else {
        println!("Handoff complete");
        println!("  Stars: {}/100 ({})", report.score.stars, report.score.grade);
        println!("  Brief: {}", brief_path.display());
        println!("  Agent: {agent_path}");
        println!("  Exit: {}", report.exit_code);
    }

    report.exit_code
}

fn resolve_url(workspace: &str, url: Option<String>, settings: &config::PlayhouseSettings) -> String {
    if let Some(u) = url {
        return u;
    }
    if let Some(u) = workspace::resolve_verify_url(workspace, settings) {
        return u;
    }
    eprintln!("[x] No URL. Pass --url or run: playhouse config set default_url http://localhost:PORT");
    std::process::exit(1);
}
