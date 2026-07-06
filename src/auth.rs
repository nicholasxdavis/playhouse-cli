use std::collections::HashMap;

use serde_json::{json, Value};

use crate::config_cli;
use crate::workspace::{self, parse_audit_headers, save_workspace_config};

pub fn login_headers(
    token: Option<&str>,
    header_name: Option<&str>,
    header_value: Option<&str>,
    basic_user: Option<&str>,
    basic_pass: Option<&str>,
) -> Result<HashMap<String, String>, String> {
    let mut headers = HashMap::new();

    if let Some(t) = token {
        headers.insert("Authorization".into(), format!("Bearer {t}"));
    }

    if let (Some(name), Some(value)) = (header_name, header_value) {
        headers.insert(name.to_string(), value.to_string());
    }

    if let (Some(user), Some(pass)) = (basic_user, basic_pass) {
        let encoded = base64_encode(&format!("{user}:{pass}"));
        headers.insert("Authorization".into(), format!("Basic {encoded}"));
    }

    if headers.is_empty() {
        return Err(
            "Provide --token, --header-name/--header-value, or --basic-user/--basic-pass".into(),
        );
    }

    Ok(headers)
}

pub fn save_auth_headers(workspace: &str, headers: HashMap<String, String>) -> Result<Value, String> {
    let json_str = serde_json::to_string(&headers).map_err(|e| e.to_string())?;
    parse_audit_headers(&json_str)?;
    let mut cfg = workspace::load_workspace_config(workspace);
    cfg.audit_headers = Some(headers.clone());
    save_workspace_config(workspace, &cfg).map_err(|e| e.to_string())?;
    let redacted: HashMap<String, String> = headers
        .into_iter()
        .map(|(k, v)| {
            let masked = if v.len() <= 8 {
                "***".into()
            } else {
                format!("{}…{}", &v[..4], &v[v.len().saturating_sub(2)..])
            };
            (k, masked)
        })
        .collect();
    Ok(json!({
        "saved": true,
        "auditHeaders": redacted,
        "configPath": workspace::workspace_config_path(workspace),
    }))
}

pub fn set_default_url_if_provided(workspace: &str, url: Option<&str>) -> Result<(), String> {
    if let Some(u) = url {
        config_cli::set(workspace, "default_url", u)?;
    }
    Ok(())
}

fn base64_encode(input: &str) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}
