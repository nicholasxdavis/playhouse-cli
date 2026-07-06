use crate::cli::context::Context;
use crate::cli::output;
use crate::verify_progress;

pub fn run(ctx: &Context<'_>) -> i32 {
    let out = verify_progress::status_json(ctx.workspace);
    if ctx.json {
        output::print_json(&out);
    } else if out["active"].as_bool() == Some(true) {
        let step = out["progress"]["currentStep"]
            .as_str()
            .unwrap_or("?");
        let pct = out["progress"]["percentComplete"].as_u64().unwrap_or(0);
        let elapsed = out["progress"]["elapsedSecs"].as_u64().unwrap_or(0);
        println!("Verify running: {step} ({pct}%, {elapsed}s elapsed)");
    } else {
        println!("No verify run in progress");
    }
    0
}
