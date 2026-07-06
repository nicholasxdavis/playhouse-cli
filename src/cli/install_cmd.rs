use crate::cli::context::Context;
use crate::cli::output;
use crate::install;

pub async fn run(ctx: &Context<'_>, minimal: bool, full: bool) -> i32 {
    let profile = install::InstallProfile::from_flags(minimal, full);
    let report = install::ensure_profile(ctx.workspace, profile, ctx.json).await;

    if ctx.json {
        output::print_json(&report);
    } else {
        for msg in &report.messages {
            println!("[*] {msg}");
        }
        for err in &report.errors {
            eprintln!("[x] {err}");
        }
    }

    if report.errors.is_empty() { 0 } else { 5 }
}
