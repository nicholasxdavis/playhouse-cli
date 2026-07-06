use crate::cli::context::Context;
use crate::cli::output;
use crate::config;
use crate::detect;
use crate::install;
use crate::pkgmgr::PackageManager;
use crate::project;
use crate::types::{CheckStatus, HealthCheck};
use crate::workspace;

pub async fn run(ctx: &Context<'_>, resolve: bool) -> i32 {
    if ctx.settings.auto_install_tools {
        let profile = project::detect(ctx.workspace);
        let _ = install::ensure_profile(ctx.workspace, profile.install_profile(), ctx.json).await;
    }

    let mut checks = detect::run_doctor(ctx.workspace);

    if resolve {
        if let Some(msg) = try_resolve_native_bindings(ctx.workspace).await {
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

async fn try_resolve_native_bindings(workspace: &str) -> Option<String> {
    let scan = workspace::scan_root(workspace);
    if !scan.join("package.json").is_file() {
        return None;
    }
    let settings = config::load_settings();
    let pm = PackageManager::resolve(workspace, &settings.package_manager);
    let out = crate::cmd::r#async(pm.program())
        .arg("rebuild")
        .current_dir(&scan)
        .output()
        .await
        .ok()?;
    if out.status.success() {
        Some(format!("{} rebuild completed", pm.label()))
    } else {
        None
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
