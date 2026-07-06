use crate::baseplates;
use crate::cli::args::TestAction;
use crate::cli::context::Context;
use crate::cli::output;
use crate::engines;
use crate::project;

pub async fn run(ctx: &Context<'_>, action: TestAction) -> i32 {
    match action {
        TestAction::List => run_list(ctx),
        TestAction::Init { plate, force } => run_init(ctx, plate.as_deref(), force),
        TestAction::Add { plate, force } => run_add(ctx, &plate, force),
        TestAction::Run => engines::functional::run(ctx.workspace, ctx.json, false).await,
    }
}

fn run_list(ctx: &Context<'_>) -> i32 {
    let profile = project::detect(ctx.workspace);
    let plates = baseplates::list_plates(&profile);
    if ctx.json {
        output::print_json(&plates);
    } else {
        for p in &plates {
            let mark = if p.compatible { "[*]" } else { "[ ]" };
            println!(
                "{mark} {} ({}) — {} [{}]",
                p.id, p.label, p.description, p.runner
            );
        }
    }
    0
}

fn run_init(ctx: &Context<'_>, plate: Option<&str>, force: bool) -> i32 {
    match baseplates::init_plate(ctx.workspace, plate, force) {
        Ok(report) => {
            if ctx.json {
                output::print_json(&report);
            } else {
                print_baseplate_report("Applied", &report);
            }
            0
        }
        Err(e) => output::print_error(e, ctx.json),
    }
}

fn run_add(ctx: &Context<'_>, plate: &str, force: bool) -> i32 {
    match baseplates::add_plate(ctx.workspace, plate, force) {
        Ok(report) => {
            if ctx.json {
                output::print_json(&report);
            } else {
                print_baseplate_report("Added", &report);
            }
            0
        }
        Err(e) => output::print_error(e, ctx.json),
    }
}

fn print_baseplate_report(verb: &str, report: &baseplates::ScaffoldReport) {
    println!("[*] {verb} baseplate: {}", report.plate);
    for f in &report.files {
        println!("[*] Wrote {f}");
    }
    for s in &report.skipped {
        println!("[!] Skipped (exists): {s}");
    }
    if let Some(cfg) = &report.playwright_config {
        println!("[*] Playwright config: {cfg}");
    }
    if !report.manifest_path.is_empty() {
        println!("[*] Manifest: {}", report.manifest_path);
    }
}
