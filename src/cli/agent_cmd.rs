use crate::agent;
use crate::cli::args::AgentAction;
use crate::cli::context::Context;
use crate::cli::handlers;
use crate::cli::output;

pub async fn run(ctx: &Context<'_>, action: Option<AgentAction>) -> i32 {
    match action {
        None => {
            output::print_json(&agent::manifest(ctx.workspace));
            0
        }
        Some(AgentAction::Status) => {
            output::print_json(&agent::status(ctx.workspace));
            0
        }
        Some(AgentAction::Plan) => {
            output::print_json(&agent::plan(ctx.workspace));
            0
        }
        Some(AgentAction::Handoff { url }) => {
            handlers::run_agent_handoff(ctx.workspace, url.as_deref(), ctx.json, ctx.settings).await
        }
    }
}
