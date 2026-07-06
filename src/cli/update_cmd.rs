use crate::cli::context::Context;
use crate::cli::output;
use crate::update;

pub fn run(ctx: &Context<'_>) -> i32 {
    let report = update::run_update(ctx.workspace);
    if ctx.json {
        output::print_json(&report);
    } else {
        println!("Current: {}", report.current);
        if let Some(ref latest) = report.latest {
            println!("Latest:  {latest}");
        }
        if report.updated {
            println!("[*] {}", report.message);
        } else if report.update_available {
            println!("[!] {}", report.message);
            if let Some(ref err) = report.error {
                eprintln!("[x] {err}");
            }
        } else {
            println!("[*] {}", report.message);
        }
    }
    if report.updated {
        0
    } else if report.update_available && report.error.is_some() {
        1
    } else {
        0
    }
}
