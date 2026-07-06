use crate::agent;
use crate::audit;
use crate::config;
use crate::detect;
use crate::dev_server;
use crate::install;
use crate::tools;
use crate::workspace;

pub struct VerifyOptions<'a> {
    pub test_pattern: Option<&'a str>,
    pub start_server: Option<&'a str>,
    pub server_port: Option<u16>,
    pub server_timeout: u64,
}

pub async fn run_verify(
    workspace: &str,
    url: Option<&str>,
    options: &VerifyOptions<'_>,
    json: bool,
    settings: &config::PlayhouseSettings,
) -> i32 {
    let mut resolved = url.map(String::from);
    let _server_guard = if let Some(start_cmd) = options.start_server {
        let ws_cfg = workspace::load_workspace_config(workspace);
        let url_hint = resolved
            .as_deref()
            .or(ws_cfg.default_url.as_deref());
        let port = dev_server::resolve_server_port(options.server_port, url_hint, workspace);
        let cwd = workspace::scan_root(workspace);
        match dev_server::spawn_and_wait(&cwd, start_cmd, port, options.server_timeout).await {
            Ok((guard, server_url)) => {
                resolved = Some(server_url);
                Some(guard)
            }
            Err(e) => {
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "command": "verify",
                            "error": e,
                            "exitCode": 1,
                            "passed": false,
                        })
                    );
                } else {
                    eprintln!("[x] {e}");
                }
                return 1;
            }
        }
    } else {
        resolved = resolved.or_else(|| workspace::resolve_verify_url(workspace, settings));
        None
    };

    if resolved.is_none() && !json && !settings.skip_lighthouse_without_server {
        let hints = detect::port_hints(workspace);
        if hints.is_empty() {
            eprintln!(
                "[!] No URL: browser audits skipped. Set: playhouse config set default_url http://localhost:PORT"
            );
        } else {
            eprintln!(
                "[!] No URL: browser audits skipped. Start dev server or: playhouse config set default_url http://localhost:{}",
                hints[0]
            );
        }
    }

    let report = audit::run_audit(
        workspace,
        resolved.as_deref(),
        settings,
        json,
        options.test_pattern,
    )
    .await;

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

pub async fn run_agent_handoff(
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

    let report = audit::run_audit(workspace, target.as_deref(), settings, true, None).await;

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

pub fn resolve_url(
    workspace: &str,
    url: Option<String>,
    settings: &config::PlayhouseSettings,
) -> String {
    if let Some(u) = url {
        return u;
    }
    if let Some(u) = workspace::resolve_verify_url(workspace, settings) {
        return u;
    }
    eprintln!("[x] No URL. Pass --url or run: playhouse config set default_url http://localhost:PORT");
    std::process::exit(1);
}

pub fn print_last_score(path: &std::path::Path, content: &str) {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(score) = v.get("playhouseScore") {
            let stars = score["stars"].as_u64().unwrap_or(0);
            let grade = score["grade"].as_str().unwrap_or("?");
            println!("Playhouse Star Rating: {stars}/100 - {grade}");
            println!("Report: {}", path.display());
            return;
        }
    }
    println!("{content}");
}
