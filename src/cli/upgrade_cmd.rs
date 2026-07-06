use crate::cli::context::Context;
use crate::cli::output;
use crate::upgrade;

pub fn run(ctx: &Context<'_>) -> i32 {
    let report = upgrade::check();
    if ctx.json {
        output::print_json(&report);
    } else {
        print_human(&report);
    }
    0
}

fn print_human(report: &upgrade::UpgradeReport) {
    println!("Playhouse {}", report.current);
    println!("Install method: {}", report.install_method);

    if let Some(ref latest) = report.github.latest {
        let mark = if report.github.update_available {
            "[*]"
        } else {
            "[ ]"
        };
        println!("{mark} GitHub latest: v{latest} — {}", report.github.url);
    } else if let Some(ref err) = report.github.error {
        println!("[!] GitHub check failed: {err}");
    }

    if let Some(ref latest) = report.npm.latest {
        let mark = if report.npm.update_available {
            "[*]"
        } else {
            "[ ]"
        };
        println!("{mark} npm latest: v{latest} — {}", report.npm.url);
    } else if let Some(ref err) = report.npm.error {
        println!("[!] npm check failed: {err}");
    }

    if report.github.update_available || report.npm.update_available {
        println!();
        println!("Upgrade:");
        println!("  {}", report.upgrade.npm);
        println!("  {}", report.upgrade.cargo);
        println!("  Releases: {}", report.upgrade.releases);
    } else {
        println!();
        println!("[*] You are on the latest published version (or offline).");
    }
}
