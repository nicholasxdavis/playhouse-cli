use std::fs;

use serde_json::{json, Value};

use crate::audit::{self, AuditReport};
use crate::baseplates;
use crate::config::{load_settings, PlayhouseSettings};
use crate::detect;
use crate::project::{self, ProjectProfile};
use crate::score::PlayhouseScore;
use crate::tools;
use crate::types::{CheckStatus, HealthCheck};
use crate::workspace::{self, WorkspaceConfig};

pub fn manifest(workspace: &str) -> Value {
    let settings = load_settings();
    let ws_config = workspace::load_workspace_config(workspace);
    let profile = project::detect(workspace);
    let checks = detect::run_doctor(workspace);
    let stay = workspace::stay_on_track_status(workspace, &settings);
    let ph_skill = workspace::playhouse_skill_status(workspace, &settings);
    let url = workspace::resolve_verify_url(workspace, &settings);
    let last_score = load_last_score(workspace);

    json!({
        "playhouse": {
            "version": env!("CARGO_PKG_VERSION"),
            "role": "QA CLI for headless agent and CI workflows",
            "startHere": "Read .playhouse/SKILL.md if present, then run `playhouse agent --json`",
            "tui": "Optional: run `playhouse` with no args for human overview",
        },
        "workspace": workspace_block(workspace, &ws_config, &url, &profile),
        "stack": stack_block(&profile),
        "tests": baseplates::tests_block(workspace, &profile),
        "tools": tools_block(workspace, &checks, &url, &profile),
        "urls": urls_block(workspace, &settings, &ws_config, &url),
        "commands": commands_reference(),
        "recipes": recipes(workspace, &url),
        "exitCodes": exit_codes(),
        "settings": settings,
        "workspaceConfig": ws_config,
        "playhouseScore": score_block(&settings, &last_score),
        "stayOnTrack": stay,
        "playhouseSkill": ph_skill,
        "readOrder": read_order(workspace, &settings, &ws_config),
        "nextActions": next_actions(workspace, &settings, &checks, &last_score, &profile),
        "handoffChecklist": handoff_checklist(&settings, &ws_config, &profile),
        "workflow": agent_workflow(workspace, &settings, &ws_config, &profile),
        "configKeys": "Run `playhouse config schema --json` for settable keys",
    })
}

pub fn status(workspace: &str) -> Value {
    let settings = load_settings();
    let ws = workspace::load_workspace_config(workspace);
    let profile = project::detect(workspace);
    let checks = detect::run_doctor(workspace);
    let url = workspace::resolve_verify_url(workspace, &settings);
    let last_score = load_last_score(workspace);
    let pass = checks.iter().filter(|c| c.status == CheckStatus::Pass).count();
    let fail = checks.iter().filter(|c| c.status == CheckStatus::Fail).count();
    let warn = checks.iter().filter(|c| c.status == CheckStatus::Warn).count();

    json!({
        "ready": fail == 0,
        "stack": profile.stack.as_str(),
        "functionalRunner": profile.functional_runner.as_str(),
        "browserAudits": profile.browser_audits,
        "toolsPass": pass,
        "toolsWarn": warn,
        "toolsFail": fail,
        "verifyUrl": url,
        "lastScore": last_score,
        "starPassThreshold": settings.star_pass_threshold,
        "initialized": ws.initialized,
        "nextActions": next_actions(workspace, &settings, &checks, &last_score, &profile),
        "doctor": checks,
    })
}

pub fn plan(workspace: &str) -> Value {
    let settings = load_settings();
    let ws = workspace::load_workspace_config(workspace);
    let profile = project::detect(workspace);
    let url = workspace::resolve_verify_url(workspace, &settings);
    let sot = ws.stay_on_track || settings.stay_on_track_enabled;

    let mut phases = Vec::new();

    phases.push(json!({
        "phase": "start",
        "steps": start_steps(workspace, &settings, &ws, sot, &profile),
    }));

    phases.push(json!({
        "phase": "during",
        "steps": during_steps(&settings, &profile),
    }));

    phases.push(json!({
        "phase": "handoff",
        "steps": handoff_steps(workspace, &url),
    }));

    json!({
        "project": ws.project_name.clone().unwrap_or_else(|| workspace::detect_project_name(workspace)),
        "stack": profile.stack.as_str(),
        "functionalRunner": profile.functional_runner.as_str(),
        "browserAudits": profile.browser_audits,
        "verifyUrl": url,
        "phases": phases,
        "commands": recipes(workspace, &url),
    })
}

pub fn build_handoff_json(
    workspace: &str,
    settings: &PlayhouseSettings,
    audit: Option<&AuditReport>,
) -> Value {
    let ws = workspace::load_workspace_config(workspace);
    let profile = project::detect(workspace);
    let checks = detect::run_doctor(workspace);
    let url = workspace::resolve_verify_url(workspace, &settings);
    let brief_path = tools::playhouse_dir(workspace).join("BRIEF.md");
    let score_path = tools::playhouse_dir(workspace).join("reports").join("score.json");

    let mut out = json!({
        "generatedAt": unix_now(),
        "workspace": workspace_block(workspace, &ws, &url, &profile),
        "stack": stack_block(&profile),
        "doctor": checks,
        "readOrder": read_order(workspace, settings, &ws),
        "handoffChecklist": handoff_checklist(settings, &ws, &profile),
        "paths": {
            "brief": brief_path.to_string_lossy(),
            "scoreReport": score_path.to_string_lossy(),
            "agentManifest": "playhouse agent --json",
        },
        "settings": settings,
        "workspaceConfig": ws,
    });

    if let Some(report) = audit {
        out["audit"] = audit::audit_json(report);
        out["playhouseScore"] = json!(report.score);
        out["exitCode"] = json!(report.exit_code);
        out["passed"] = json!(report.exit_code == 0);
    } else if let Some(score) = load_last_score(workspace) {
        out["playhouseScore"] = json!(score);
    }

    out
}

pub fn save_handoff_json(
    workspace: &str,
    settings: &PlayhouseSettings,
    audit: Option<&AuditReport>,
) -> std::io::Result<std::path::PathBuf> {
    let path = workspace::agent_json_path(workspace);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let doc = build_handoff_json(workspace, settings, audit);
    fs::write(&path, serde_json::to_string_pretty(&doc).unwrap_or_default())?;
    Ok(path)
}

pub fn build_brief_text(
    workspace: &str,
    settings: &PlayhouseSettings,
    ws_config: &WorkspaceConfig,
) -> String {
    let profile = project::detect(workspace);
    let checks = detect::run_doctor(workspace);
    let pass = checks.iter().filter(|c| c.status == CheckStatus::Pass).count();
    let total = checks.len();
    let url = workspace::resolve_verify_url(workspace, settings)
        .unwrap_or_else(|| "none".into());
    let project = ws_config
        .project_name
        .clone()
        .unwrap_or_else(|| workspace::detect_project_name(workspace));
    let skill = workspace::skill_path(workspace, settings);
    let ph_skill = workspace::playhouse_skill_path(workspace, settings);
    let last_score = load_last_score(workspace);
    let stars_line = last_score
        .as_ref()
        .map(|s| format!("Last score: {}/100 ({})", s.stars, s.grade))
        .unwrap_or_else(|| "Last score: none (run playhouse verify)".into());

    let lh = settings.lighthouse_threshold * 100.0;
    let sot = if ws_config.stay_on_track || settings.stay_on_track_enabled {
        "enabled"
    } else {
        "disabled"
    };
    let ph = if ws_config.playhouse_skill || settings.playhouse_skill_enabled {
        "enabled (recommended)"
    } else {
        "disabled"
    };
    let notes = ws_config
        .agent_notes
        .as_deref()
        .unwrap_or("none");

    format!(
        r#"# Playhouse Workspace Brief

Project: {project}
Workspace: {workspace}
Verify URL: {url}
Stack: {stack} | Functional runner: {runner} | Browser audits: {browser}
Tools ready: {pass}/{total}
{stars_line}
Stars pass threshold: {star_threshold}/100
Package manager: {pkg_mgr}
Lighthouse threshold: {lh:.0}%
Trivy severity: {sev}
Stay-on-track: {sot}
Playhouse skill: {ph} (`{ph_skill}`)
Agent notes: {notes}

## Agent workflow

1. `playhouse agent --json` - full manifest (read first)
2. `playhouse agent status --json` - quick health check
3. `playhouse agent plan --json` - phased workflow for this repo
4. `playhouse agent handoff --json` - run verify and export handoff bundle
5. `playhouse config schema --json` - all settable keys

## Headless commands

```bash
playhouse doctor --json
playhouse install
playhouse init [--stay-on-track]
playhouse verify [--url URL] --json
playhouse score [--url URL] [--last] --json
playhouse playwright [pattern] --json
playhouse trivy --json
playhouse arkenar [url] --json
playhouse lighthouse [url] --json
playhouse config get|set <key> [value] --json
playhouse export
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Pass |
| 1 | Test or verify failure |
| 2 | Lighthouse below threshold |
| 3 | Arkenar high/medium findings |
| 4 | Trivy findings |
| 5 | Tool missing - run playhouse install |

## Handoff checklist

1. Run `playhouse verify --json` or `playhouse agent handoff --json`
2. Fix Playwright failures, Arkenar findings, Trivy HIGH/CRITICAL
3. Lighthouse scores above {lh:.0}%
4. Playhouse Stars at or above {star_threshold}/100
5. Never commit secrets
6. If stay-on-track enabled, complete `{skill}` first

## Config files

- Global: `~/.config/playhouse/settings.json` (or platform equivalent)
- Workspace: `.playhouse/config.json`
- Handoff bundle: `.playhouse/AGENT.json`
- Score report: `.playhouse/reports/score.json`
"#,
        project = project,
        workspace = workspace,
        url = url,
        stack = profile.stack.as_str(),
        runner = profile.functional_runner.as_str(),
        browser = profile.browser_audits,
        pass = pass,
        total = total,
        stars_line = stars_line,
        star_threshold = settings.star_pass_threshold,
        pkg_mgr = settings.package_manager,
        lh = lh,
        sev = settings.trivy_severity,
        sot = sot,
        ph = ph,
        ph_skill = ph_skill.display(),
        notes = notes,
        skill = skill.display(),
    )
}

fn stack_block(profile: &ProjectProfile) -> Value {
    json!({
        "stack": profile.stack.as_str(),
        "functionalRunner": profile.functional_runner.as_str(),
        "browserAudits": profile.browser_audits,
        "language": profile.language,
        "signals": profile.signals,
    })
}

fn workspace_block(
    workspace: &str,
    ws: &WorkspaceConfig,
    url: &Option<String>,
    profile: &ProjectProfile,
) -> Value {
    let roots = workspace::resolve_roots(workspace);
    json!({
        "path": workspace,
        "projectName": ws.project_name.clone().unwrap_or_else(|| workspace::detect_project_name(workspace)),
        "scanRoot": ws.scan_root,
        "testRoot": ws.test_root,
        "resolvedScanRoot": roots.scan.to_string_lossy(),
        "resolvedTestRoot": roots.test.to_string_lossy(),
        "functionalRunnerOverride": ws.functional_runner,
        "stack": profile.stack.as_str(),
        "functionalRunner": profile.functional_runner.as_str(),
        "browserAudits": profile.browser_audits,
        "language": profile.language,
        "signals": profile.signals,
        "playhouseDir": tools::playhouse_dir(workspace).to_string_lossy(),
        "briefPath": ".playhouse/BRIEF.md",
        "agentPath": ".playhouse/AGENT.json",
        "advisoriesPath": ".playhouse/advisories.log",
        "configPath": ".playhouse/config.json",
        "scoreReportPath": ".playhouse/reports/score.json",
        "initialized": ws.initialized,
        "defaultUrl": ws.default_url,
        "verifyUrl": url,
        "agentNotes": ws.agent_notes,
    })
}

fn tools_block(
    workspace: &str,
    checks: &[HealthCheck],
    url: &Option<String>,
    profile: &ProjectProfile,
) -> Value {
    let settings = load_settings();
    json!({
        "bundledTrivy": tools::has_bundled_trivy(),
        "trivyPath": tools::trivy_program(),
        "playwrightPrefix": tools::playwright_prefix(workspace).to_string_lossy(),
        "playwrightInstalled": tools::has_playwright(workspace),
        "packageManager": {
            "setting": settings.package_manager,
            "resolved": crate::pkgmgr::PackageManager::resolve(workspace, &settings.package_manager).id(),
        },
        "arkenarPath": tools::arkenar_program(),
        "arkenarInstalled": tools::has_bundled_arkenar(),
        "stack": profile.stack.as_str(),
        "functionalRunner": profile.functional_runner.as_str(),
        "browserAudits": profile.browser_audits,
        "localServer": url.clone(),
        "doctor": checks,
    })
}

fn urls_block(workspace: &str, settings: &PlayhouseSettings, ws: &WorkspaceConfig, resolved: &Option<String>) -> Value {
    let hints = detect::port_hints(workspace);
    let suggested = detect::suggested_local_url(workspace);
    json!({
        "resolved": resolved,
        "workspaceDefault": ws.default_url,
        "globalDefault": settings.default_lighthouse_url,
        "detectedLocal": detect::find_local_server(workspace),
        "portHints": hints,
        "suggestedUrl": suggested,
        "priority": "workspace default_url (playhouse config set default_url) > global default_lighthouse_url > live local server (port hints from package.json/vite/wrangler, then common ports)",
        "setUrl": "playhouse config set default_url http://localhost:PORT",
    })
}

fn score_block(settings: &PlayhouseSettings, last: &Option<PlayhouseScore>) -> Value {
    json!({
        "scale": "0-100",
        "passThreshold": settings.star_pass_threshold,
        "reportPath": ".playhouse/reports/score.json",
        "methodology": crate::score::METHODOLOGY,
        "last": last,
    })
}

fn read_order(workspace: &str, settings: &PlayhouseSettings, ws: &WorkspaceConfig) -> Vec<Value> {
    let mut order = Vec::new();
    let mut step = 1;

    if ws.playhouse_skill || settings.playhouse_skill_enabled {
        let skill = workspace::playhouse_skill_path(workspace, settings);
        order.push(json!({
            "step": step,
            "action": "read",
            "target": skill.to_string_lossy(),
            "why": "Playhouse agent skill (recommended) - how to use this CLI",
        }));
        step += 1;
    }

    order.push(json!({ "step": step, "action": "run", "target": "playhouse agent --json", "why": "Full manifest and workflow" }));
    step += 1;
    order.push(json!({ "step": step, "action": "read", "target": ".playhouse/BRIEF.md", "why": "Workspace QA summary" }));
    step += 1;

    if ws.stay_on_track || settings.stay_on_track_enabled {
        let skill = workspace::skill_path(workspace, settings);
        let project = workspace::project_info_path(workspace, settings);
        order.push(json!({ "step": step, "action": "read", "target": skill.to_string_lossy(), "why": "Stay-on-track skill rules" }));
        step += 1;
        order.push(json!({ "step": step, "action": "read", "target": project.to_string_lossy(), "why": "Complete project info with user" }));
        step += 1;
    }
    if let Some(notes) = &ws.agent_notes {
        order.push(json!({ "step": step, "action": "note", "target": notes, "why": "Workspace agent notes" }));
        step += 1;
    }
    order.push(json!({ "step": step, "action": "run", "target": "playhouse doctor --json", "why": "Confirm tools before QA" }));
    step += 1;
    if !baseplates::detect_existing_tests(workspace).detected {
        if let Some(plate) = baseplates::default_plate_for_profile(&project::detect(workspace)) {
            order.push(json!({
                "step": step,
                "action": "run",
                "target": format!("playhouse test init --plate {plate} --json"),
                "why": "Scaffold starter functional tests when none exist",
            }));
            step += 1;
        }
    }
    order.push(json!({ "step": step, "action": "run", "target": "playhouse verify --json", "why": "Full audit before handoff" }));
    order
}

fn next_actions(
    workspace: &str,
    settings: &PlayhouseSettings,
    checks: &[HealthCheck],
    last_score: &Option<PlayhouseScore>,
    profile: &ProjectProfile,
) -> Vec<Value> {
    let mut actions = Vec::new();

    if !workspace::load_workspace_config(workspace).initialized {
        actions.push(json!({
            "priority": "high",
            "action": "playhouse init --json",
            "reason": "Workspace not initialized",
        }));
    }

    if checks.iter().any(|c| c.name.contains("Trivy") && c.status != CheckStatus::Pass)
        || (profile.needs_playwright()
            && checks
                .iter()
                .any(|c| c.name.contains("Playwright") && c.status == CheckStatus::Warn))
    {
        let install_cmd = if profile.browser_audits || profile.needs_playwright() {
            "playhouse install --full"
        } else {
            "playhouse install --minimal"
        };
        actions.push(json!({
            "priority": "high",
            "action": install_cmd,
            "reason": "Bundled tools missing or incomplete",
        }));
    }

    if checks.iter().any(|c| c.status == CheckStatus::Fail) {
        actions.push(json!({
            "priority": "high",
            "action": "playhouse doctor --json",
            "reason": "Tool health failures detected",
        }));
    }

    let detection = baseplates::detect_existing_tests(workspace);
    if !detection.detected {
        if let Some(plate) = baseplates::default_plate_for_profile(profile) {
            actions.push(json!({
                "priority": "medium",
                "action": format!("playhouse test init --plate {plate} --json"),
                "reason": "No functional tests detected — scaffold a starter baseplate",
            }));
        }
    }

    if profile.browser_audits && workspace::resolve_verify_url(workspace, settings).is_none() {
        actions.push(json!({
            "priority": "medium",
            "action": "playhouse config set default_url <url>",
            "reason": "No verify URL - set workspace default or start dev server",
        }));
    }

    match last_score {
        None => actions.push(json!({
            "priority": "medium",
            "action": "playhouse verify --json",
            "reason": "No score report yet",
        })),
        Some(s) if !s.passed => actions.push(json!({
            "priority": "high",
            "action": "playhouse verify --json",
            "reason": format!("Last score {}/100 below pass threshold", s.stars),
        })),
        Some(s) => actions.push(json!({
            "priority": "low",
            "action": "playhouse agent handoff --json",
            "reason": format!("Last score {}/100 passed - ready for handoff refresh", s.stars),
        })),
    }

    if actions.is_empty() {
        actions.push(json!({
            "priority": "low",
            "action": "playhouse agent handoff --json",
            "reason": "All checks look good",
        }));
    }

    actions
}

fn handoff_checklist(
    settings: &PlayhouseSettings,
    ws: &WorkspaceConfig,
    profile: &ProjectProfile,
) -> Vec<Value> {
    let sot = ws.stay_on_track || settings.stay_on_track_enabled;
    let ph = ws.playhouse_skill || settings.playhouse_skill_enabled;
    let needs_pw = profile.needs_playwright() && !settings.skip_playwright_in_verify;
    let mut items = vec![
        json!({ "id": "playhouse_skill", "task": "Read .playhouse/SKILL.md (recommended)", "required": ph }),
        json!({ "id": "doctor", "task": "Tools healthy (playhouse doctor --json)", "required": true }),
        json!({ "id": "verify", "task": "Full verify passed (playhouse verify --json)", "required": true }),
        json!({ "id": "stars", "task": format!("Playhouse Stars >= {}/100", settings.star_pass_threshold), "required": true }),
        json!({ "id": "secrets", "task": "No Trivy secrets or HIGH/CRITICAL vulns", "required": true }),
        json!({ "id": "playwright", "task": "All Playwright tests pass", "required": needs_pw }),
        json!({ "id": "brief", "task": "BRIEF.md exported (.playhouse/BRIEF.md)", "required": false }),
        json!({ "id": "agent_json", "task": "AGENT.json handoff bundle (.playhouse/AGENT.json)", "required": settings.auto_export_handoff_json }),
    ];
    if sot {
        items.push(json!({ "id": "stay_on_track", "task": ".playhouse/stay-on-track skill and PROJECT.md complete", "required": true }));
    }
    items
}

fn start_steps(
    workspace: &str,
    settings: &PlayhouseSettings,
    ws: &WorkspaceConfig,
    sot: bool,
    profile: &ProjectProfile,
) -> Vec<&'static str> {
    let ph = ws.playhouse_skill || settings.playhouse_skill_enabled;
    let mut steps = Vec::new();
    if ph {
        steps.push("Read .playhouse/SKILL.md (recommended)");
    }
    steps.push("playhouse agent --json");
    steps.push("Read .playhouse/BRIEF.md");
    if !ws.initialized {
        steps.push("playhouse init --json");
    }
    if sot {
        steps.push("Read .playhouse/stay-on-track/SKILL.md and complete PROJECT.md with user");
    }
    steps.push("playhouse doctor --json");
    if !tools::has_bundled_trivy()
        || (profile.needs_playwright() && !tools::has_playwright(workspace))
    {
        steps.push(if profile.browser_audits || profile.needs_playwright() {
            "playhouse install --full"
        } else {
            "playhouse install --minimal"
        });
    }
    let _ = settings;
    steps
}

fn during_steps(settings: &PlayhouseSettings, profile: &ProjectProfile) -> Vec<&'static str> {
    let mut steps = vec!["Make changes", "Run targeted checks as needed"];
    if profile.functional_runner != project::FunctionalRunner::None {
        steps.push("playhouse test run --json");
    }
    if !settings.skip_playwright_in_verify && profile.needs_playwright() {
        steps.push("playhouse playwright --json");
    }
    if !settings.skip_trivy_in_verify {
        steps.push("playhouse trivy --json");
    }
    steps
}

fn handoff_steps(_workspace: &str, url: &Option<String>) -> Vec<String> {
    let mut steps = vec![
        "playhouse verify --json".into(),
        "Fix all failures and re-run until exit 0".into(),
        "playhouse agent handoff --json".into(),
        "playhouse export".into(),
    ];
    if url.is_none() {
        steps.insert(0, "Set URL: playhouse config set default_url http://localhost:PORT".into());
    }
    steps
}

fn recipes(_workspace: &str, url: &Option<String>) -> Value {
    let url_flag = url
        .as_ref()
        .map(|u| format!(" --url {u}"))
        .unwrap_or_default();
    json!({
        "bootstrap": [
            "playhouse agent --json",
            "playhouse init --json",
            "playhouse install",
            "playhouse doctor --json",
        ],
        "quickCheck": [
            "playhouse agent status --json",
            "playhouse doctor --json",
        ],
        "functional": [
            "playhouse test list --json",
            "playhouse test init --plate web-smoke --json",
            "playhouse test add --plate web-a11y --json",
            "playhouse test run --json",
            "playhouse functional --json",
        ],
        "fullAudit": [
            format!("playhouse verify{url_flag} --json"),
            "playhouse score --last --json",
        ],
        "handoff": [
            format!("playhouse agent handoff{url_flag} --json"),
            "playhouse export",
        ],
        "config": [
            "playhouse config schema --json",
            "playhouse config get package_manager",
            "playhouse config set default_url http://localhost:3000",
        ],
    })
}

fn commands_reference() -> Value {
    json!([
        { "cmd": "playhouse agent [--json]", "desc": "Full agent manifest" },
        { "cmd": "playhouse agent status [--json]", "desc": "Quick health and next actions" },
        { "cmd": "playhouse agent plan [--json]", "desc": "Phased workflow for this workspace" },
        { "cmd": "playhouse agent handoff [--url URL] [--json]", "desc": "Run verify and write handoff bundle" },
        { "cmd": "playhouse config [--json]", "desc": "Show all settings" },
        { "cmd": "playhouse config schema [--json]", "desc": "List settable config keys" },
        { "cmd": "playhouse config get <key>", "desc": "Read a setting" },
        { "cmd": "playhouse config set <key> <value>", "desc": "Update a setting" },
        { "cmd": "playhouse init [--stay-on-track] [--json]", "desc": "Set up .playhouse/" },
        { "cmd": "playhouse install [--json]", "desc": "Install bundled tools" },
        { "cmd": "playhouse doctor [--json]", "desc": "Tool health check" },
        { "cmd": "playhouse verify [--url URL] [--json]", "desc": "Full QA + Playhouse Stars" },
        { "cmd": "playhouse score [--url URL] [--last] [--json]", "desc": "Star rating audit" },
        { "cmd": "playhouse functional [--json]", "desc": "Run detected functional test runner" },
        { "cmd": "playhouse test list [--json]", "desc": "List scaffold baseplates for this stack" },
        { "cmd": "playhouse test init [--plate ID] [--force] [--json]", "desc": "Scaffold starter tests when none exist" },
        { "cmd": "playhouse test add --plate ID [--force] [--json]", "desc": "Add another baseplate file" },
        { "cmd": "playhouse test run [--json]", "desc": "Run functional tests (same as functional)" },
        { "cmd": "playhouse playwright [pattern] [--json]", "desc": "Playwright tests (web E2E)" },
        { "cmd": "playhouse trivy [--json]", "desc": "Security and secret scan" },
        { "cmd": "playhouse arkenar [url] [--json]", "desc": "DAST web scan" },
        { "cmd": "playhouse lighthouse [url] [--json]", "desc": "Performance audit" },
        { "cmd": "playhouse skill install|enable|disable|status [--json]", "desc": ".playhouse/SKILL.md agent skill (recommended)" },
        { "cmd": "playhouse stay-on-track enable|disable|status [--json]", "desc": "Optional agent discipline skill" },
        { "cmd": "playhouse export [--json]", "desc": "Write BRIEF.md" },
        { "cmd": "playhouse upgrade [--json]", "desc": "Check GitHub Releases and npm for updates" },
    ])
}

fn exit_codes() -> Value {
    json!({
        "0": "success",
        "1": "test or verify failure",
        "2": "lighthouse below threshold",
        "3": "arkenar DAST findings",
        "4": "trivy vulnerabilities or secrets",
        "5": "required tool missing - run playhouse install",
    })
}

fn agent_workflow(
    workspace: &str,
    settings: &PlayhouseSettings,
    ws: &WorkspaceConfig,
    profile: &ProjectProfile,
) -> Value {
    let sot = ws.stay_on_track || settings.stay_on_track_enabled;
    json!({
        "phases": {
            "start": start_steps(workspace, settings, ws, sot, profile),
            "during": during_steps(settings, profile),
            "handoff": handoff_steps(workspace, &workspace::resolve_verify_url(workspace, settings)),
        },
        "playhouseSkillRecommended": true,
        "stayOnTrackOptional": true,
        "agentMode": settings.agent_mode,
    })
}

fn load_last_score(workspace: &str) -> Option<PlayhouseScore> {
    let path = tools::playhouse_dir(workspace).join("reports").join("score.json");
    let content = fs::read_to_string(path).ok()?;
    let v: Value = serde_json::from_str(&content).ok()?;
    serde_json::from_value(v.get("playhouseScore")?.clone()).ok()
}

fn unix_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}
