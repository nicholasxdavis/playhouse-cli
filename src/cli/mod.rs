mod agent_cmd;
mod args;
mod audit;
mod config_cmd;
mod context;
mod doctor;
mod handlers;
mod install_cmd;
mod output;
mod test_cmd;
mod upgrade_cmd;
mod workspace_cmd;

pub use args::Cli;

use args::Commands;
use context::Context;

use crate::config;
use crate::tui;
use crate::workspace;

pub async fn run(cli: Cli) -> i32 {
    let workspace = resolve_workspace(cli.workspace.as_deref());

    let settings = config::load_settings();
    workspace::maybe_auto_init(&workspace, &settings);

    let json = cli.json || settings.json_output_default || settings.agent_mode;
    let ctx = Context {
        workspace: &workspace,
        settings: &settings,
        json,
    };

    match cli.command {
        None => tui::run(&workspace).await,

        Some(Commands::Doctor) => doctor::run(&ctx).await,

        Some(Commands::Install { minimal, full }) => {
            install_cmd::run(&ctx, minimal, full).await
        }

        Some(Commands::Init { stay_on_track }) => workspace_cmd::run_init(&ctx, stay_on_track).await,

        Some(Commands::Agent { action }) => agent_cmd::run(&ctx, action).await,

        Some(Commands::Config { action }) => config_cmd::run(&ctx, action),

        Some(Commands::Export) => workspace_cmd::run_export(&ctx),

        Some(Commands::StayOnTrack { action }) => workspace_cmd::run_stay_on_track(&ctx, action),

        Some(Commands::Skill { action }) => workspace_cmd::run_skill(&ctx, action),

        Some(Commands::Lighthouse { url }) => audit::run_lighthouse(&ctx, url).await,

        Some(Commands::Playwright { pattern }) => audit::run_playwright(&ctx, pattern).await,

        Some(Commands::Functional { pattern }) => audit::run_functional(&ctx, pattern).await,

        Some(Commands::Test { action }) => test_cmd::run(&ctx, action).await,

        Some(Commands::Trivy) => audit::run_trivy(&ctx).await,

        Some(Commands::Arkenar { url }) => audit::run_arkenar(&ctx, url).await,

        Some(Commands::Verify {
            url,
            test,
            start_server,
            server_port,
            server_timeout,
        }) => audit::run_verify(
            &ctx,
            url,
            test,
            start_server,
            server_port,
            server_timeout,
        )
        .await,

        Some(Commands::Score { url, last }) => audit::run_score(&ctx, url, last).await,

        Some(Commands::Upgrade) => upgrade_cmd::run(&ctx),
    }
}

fn resolve_workspace(flag: Option<&str>) -> String {
    if let Some(path) = flag {
        let p = std::path::Path::new(path);
        if p.is_absolute() {
            return p.to_string_lossy().into_owned();
        }
        if let Ok(cwd) = std::env::current_dir() {
            return cwd.join(p).to_string_lossy().into_owned();
        }
        return path.to_string();
    }
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .to_string_lossy()
        .to_string()
}
