use crate::agent;
use crate::cli::args::{SkillAction, StayOnTrackAction};
use crate::cli::context::Context;
use crate::cli::output;
use crate::tools;
use crate::workspace;

pub async fn run_init(ctx: &Context<'_>, stay_on_track: bool, no_skill: bool) -> i32 {
    let enable_sot = stay_on_track || ctx.settings.stay_on_track_enabled;
    match workspace::init_workspace(
        ctx.workspace,
        ctx.settings,
        true,
        enable_sot,
        !no_skill,
        ctx.json,
    )
    .await
    {
        Ok(report) => {
            if ctx.json {
                output::print_json(&report);
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
        Err(e) => output::print_error(e, ctx.json),
    }
}

pub fn run_export(ctx: &Context<'_>) -> i32 {
    let ws = workspace::load_workspace_config(ctx.workspace);
    let brief = agent::build_brief_text(ctx.workspace, ctx.settings, &ws);
    let path = tools::playhouse_dir(ctx.workspace).join("BRIEF.md");
    let _ = std::fs::create_dir_all(path.parent().unwrap());
    match std::fs::write(&path, &brief) {
        Ok(()) => {
            if ctx.json {
                output::print_json(&serde_json::json!({ "path": path, "written": true }));
            } else {
                println!("[*] Exported {}", path.display());
            }
            0
        }
        Err(e) => output::print_error(e, ctx.json),
    }
}

pub fn run_stay_on_track(ctx: &Context<'_>, action: StayOnTrackAction) -> i32 {
    match action {
        StayOnTrackAction::Enable => match workspace::enable_stay_on_track_mode(ctx.workspace, ctx.settings) {
            Ok(path) => {
                if ctx.json {
                    output::print_json(&serde_json::json!({
                        "enabled": true,
                        "skillPath": path,
                    }));
                } else {
                    println!("[*] Stay-on-track enabled: {}", path.display());
                }
                0
            }
            Err(e) => output::print_error(e, ctx.json),
        },
        StayOnTrackAction::Disable => match workspace::disable_stay_on_track_mode(ctx.workspace) {
            Ok(()) => {
                if ctx.json {
                    println!(r#"{{"enabled":false}}"#);
                } else {
                    println!("[*] Stay-on-track disabled for workspace");
                }
                0
            }
            Err(e) => output::print_error(e, ctx.json),
        },
        StayOnTrackAction::Status => {
            output::print_json(&workspace::stay_on_track_status(ctx.workspace, ctx.settings));
            0
        }
    }
}

pub fn run_skill(ctx: &Context<'_>, action: SkillAction) -> i32 {
    match action {
        SkillAction::Install | SkillAction::Enable => {
            match workspace::install_playhouse_skill(ctx.workspace, ctx.settings) {
                Ok(path) => {
                    if ctx.json {
                        output::print_json(&serde_json::json!({
                            "enabled": true,
                            "recommended": true,
                            "skillPath": path,
                        }));
                    } else {
                        println!("[*] Playhouse agent skill: {}", path.display());
                    }
                    0
                }
                Err(e) => output::print_error(e, ctx.json),
            }
        }
        SkillAction::Disable => match workspace::disable_playhouse_skill_mode(ctx.workspace) {
            Ok(()) => {
                if ctx.json {
                    println!(r#"{{"enabled":false}}"#);
                } else {
                    println!("[*] Playhouse agent skill disabled for workspace");
                }
                0
            }
            Err(e) => output::print_error(e, ctx.json),
        },
        SkillAction::Status => {
            output::print_json(&workspace::playhouse_skill_status(ctx.workspace, ctx.settings));
            0
        }
    }
}
