use crate::cli::args::ConfigAction;
use crate::cli::context::Context;
use crate::cli::output;
use crate::config;
use crate::config_cli;
use crate::tools;
use crate::workspace;

pub fn run(ctx: &Context<'_>, action: Option<ConfigAction>) -> i32 {
    match action {
        None => {
            let ws = workspace::load_workspace_config(ctx.workspace);
            output::print_json(&serde_json::json!({
                "global": ctx.settings,
                "workspace": ws,
                "paths": {
                    "playhouseHome": config::playhouse_home(),
                    "settingsFile": config::settings_path(),
                    "workspaceConfig": workspace::workspace_config_path(ctx.workspace),
                    "bundledTrivy": tools::bundled_trivy_path(),
                    "playwrightPrefix": tools::playwright_prefix(ctx.workspace),
                    "agentHandoff": workspace::agent_json_path(ctx.workspace),
                },
                "schema": "playhouse config schema --json",
            }));
            0
        }
        Some(ConfigAction::Schema) => {
            output::print_json(&config_cli::schema_json());
            0
        }
        Some(ConfigAction::Get { key }) => match config_cli::get(ctx.workspace, &key) {
            Ok(v) => {
                if ctx.json {
                    output::print_json(&serde_json::json!({ "key": key, "value": v }));
                } else {
                    println!("{key} = {}", serde_json::to_string(&v).unwrap_or_default());
                }
                0
            }
            Err(e) => output::print_error(e, ctx.json),
        },
        Some(ConfigAction::Set { key, value }) => match config_cli::set(ctx.workspace, &key, &value) {
            Ok(v) => {
                if ctx.json {
                    output::print_json(&serde_json::json!({ "key": key, "value": v, "saved": true }));
                } else {
                    println!("[*] Set {key} = {}", serde_json::to_string(&v).unwrap_or_default());
                }
                0
            }
            Err(e) => output::print_error(e, ctx.json),
        },
    }
}
