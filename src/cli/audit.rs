use crate::audit;
use crate::cli::context::Context;
use crate::cli::handlers;
use crate::cli::output;
use crate::engines;
use crate::project::{self, FunctionalRunner};
use crate::tools;
use crate::workspace;

pub async fn run_lighthouse(ctx: &Context<'_>, url: Option<String>) -> i32 {
    let target = handlers::resolve_url(ctx.workspace, url, ctx.settings);
    engines::lighthouse::run(&target, ctx.workspace, ctx.json, false).await
}

pub async fn run_playwright(ctx: &Context<'_>, pattern: Option<String>) -> i32 {
    let profile = project::detect(ctx.workspace);
    if profile.functional_runner != FunctionalRunner::Playwright {
        return engines::functional::run(ctx.workspace, ctx.json, false).await;
    }
    engines::playwright::run(ctx.workspace, pattern.as_deref(), ctx.json, false).await
}

pub async fn run_functional(ctx: &Context<'_>) -> i32 {
    engines::functional::run(ctx.workspace, ctx.json, false).await
}

pub async fn run_trivy(ctx: &Context<'_>) -> i32 {
    engines::trivy::run(ctx.workspace, ctx.json, false).await
}

pub async fn run_arkenar(ctx: &Context<'_>, url: Option<String>) -> i32 {
    let target = handlers::resolve_url(ctx.workspace, url, ctx.settings);
    engines::arkenar::run(&target, ctx.workspace, ctx.json, false).await
}

pub async fn run_verify(ctx: &Context<'_>, url: Option<String>) -> i32 {
    let target = url.or_else(|| workspace::resolve_verify_url(ctx.workspace, ctx.settings));
    handlers::run_verify(ctx.workspace, target.as_deref(), ctx.json, ctx.settings).await
}

pub async fn run_score(ctx: &Context<'_>, url: Option<String>, last: bool) -> i32 {
    if last {
        return print_last_score(ctx);
    }

    let target = url.or_else(|| workspace::resolve_verify_url(ctx.workspace, ctx.settings));
    let report = audit::run_audit(ctx.workspace, target.as_deref(), ctx.settings, ctx.json).await;
    if ctx.json {
        output::print_json(&audit::audit_json(&report));
    }
    report.exit_code
}

fn print_last_score(ctx: &Context<'_>) -> i32 {
    let path = tools::playhouse_dir(ctx.workspace)
        .join("reports")
        .join("score.json");
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            if ctx.json {
                println!("{content}");
            } else {
                handlers::print_last_score(&path, &content);
            }
            0
        }
        Err(_) => {
            if ctx.json {
                output::print_json(&serde_json::json!({
                    "error": "no score report - run playhouse score or verify first"
                }));
            } else {
                eprintln!("[x] No score report - run `playhouse score` or `playhouse verify` first");
            }
            1
        }
    }
}
