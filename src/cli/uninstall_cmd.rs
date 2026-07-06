use crate::cli::context::Context;
use crate::cli::output;
use crate::uninstall;

pub async fn run(ctx: &Context<'_>, global: bool, workspace_tools: bool, yes: bool) -> i32 {
    let remove_global = global || !workspace_tools;
    let remove_ws = workspace_tools || !global;

    if let Err(e) = uninstall::confirm(yes) {
        return output::print_error(e, ctx.json);
    }

    let report = uninstall::uninstall_all(ctx.workspace, remove_global, remove_ws);

    if ctx.json {
        output::print_json(&report);
    } else {
        for path in &report.removed {
            println!("[*] Removed {path}");
        }
        for err in &report.failed {
            eprintln!("[x] {err}");
        }
        if report.removed.is_empty() && report.failed.is_empty() {
            println!("[*] Nothing to remove");
        }
    }

    if report.failed.is_empty() { 0 } else { 1 }
}
