use crate::auth;
use crate::cli::args::AuthAction;
use crate::cli::context::Context;
use crate::cli::output;

pub fn run(ctx: &Context<'_>, action: AuthAction) -> i32 {
    match action {
        AuthAction::Login {
            url,
            token,
            header_name,
            header_value,
            basic_user,
            basic_pass,
        } => {
            let headers = match auth::login_headers(
                token.as_deref(),
                header_name.as_deref(),
                header_value.as_deref(),
                basic_user.as_deref(),
                basic_pass.as_deref(),
            ) {
                Ok(h) => h,
                Err(e) => return output::print_error(e, ctx.json),
            };
            if let Err(e) = auth::set_default_url_if_provided(ctx.workspace, url.as_deref()) {
                return output::print_error(e, ctx.json);
            }
            match auth::save_auth_headers(ctx.workspace, headers) {
                Ok(v) => {
                    if ctx.json {
                        output::print_json(&v);
                    } else {
                        println!("[*] Saved audit headers to workspace config");
                        if let Some(path) = v.get("configPath").and_then(|p| p.as_str()) {
                            println!("[*] Config: {path}");
                        }
                    }
                    0
                }
                Err(e) => output::print_error(e, ctx.json),
            }
        }
    }
}
