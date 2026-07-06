use crate::cli::context::Context;
use crate::cli::output;
use crate::detect;
use crate::install;
use crate::project;
use crate::types::{CheckStatus, HealthCheck};

pub async fn run(ctx: &Context<'_>, resolve: bool) -> i32 {
    if ctx.settings.auto_install_tools {
        let profile = project::detect(ctx.workspace);
        let _ = install::ensure_profile(ctx.workspace, profile.install_profile(), ctx.json).await;
    }

    let mut checks = detect::run_doctor(ctx.workspace);

    if resolve {
        if let Some(msg) = detect::resolve_native_bindings(ctx.workspace).await {
            checks.push(HealthCheck::pass("Native binding resolve", &msg));
        }
    }

    print_doctor(&checks, ctx.json);

    if checks.iter().any(|c| matches!(c.status, CheckStatus::Fail)) {
        1
    } else {
        0
    }
}

fn print_doctor(checks: &[HealthCheck], json: bool) {
    if json {
        output::print_json(checks);
    } else {
        for check in checks {
            println!("{} {} -- {}", check.icon(), check.name, check.detail);
        }
    }
}
