use serde_json::{json, Value};

/// How agents should run Playhouse without shell-specific pitfalls.
pub fn support_block(workspace: &str) -> Value {
    let bad = format!(r#"cd "{workspace}" && playhouse doctor --json"#);
    let fixed = rewrite_cd_playhouse(&bad, workspace)
        .unwrap_or_else(|| format!(r#"playhouse -C "{workspace}" doctor --json"#));
    json!({
        "preferDirectCommands": true,
        "avoidChaining": "Do not combine cd with playhouse using && in PowerShell 5.x",
        "workspaceFlag": {
            "flag": "-C",
            "long": "--workspace",
            "example": format!("playhouse -C \"{workspace}\" doctor --json"),
            "why": "Sets the project root without cd or shell chaining",
        },
        "statementSeparators": {
            "posix": ["&&", "||", ";"],
            "powershell5": [";", "|"],
            "powershell7": ["&&", "||", ";"],
            "cmd": ["&&", "||", "&"],
        },
        "conflicts": [
            {
                "pattern": "cd PATH && playhouse",
                "issue": "&& is invalid in Windows PowerShell 5.x",
                "fix": format!("playhouse -C \"PATH\" <command> --json"),
            },
            {
                "pattern": "cd PATH; playhouse",
                "issue": "Works in PowerShell but changes shell cwd for later commands",
                "fix": "Use playhouse -C PATH or set the shell tool working_directory",
            },
        ],
        "recommended": {
            "cursorShell": "Set working_directory to the project root, then run playhouse commands with no cd prefix",
            "powershell5": format!("playhouse -C \"{workspace}\" agent status --json"),
            "posix": format!("playhouse -C \"{workspace}\" agent status --json"),
        },
        "playhouseCommands": "Never prefix playhouse with cd. Use -C/--workspace or the shell cwd.",
        "translateExample": {
            "from": bad,
            "to": fixed,
            "powershell5": translate_line(&bad, ShellTarget::PowerShell5),
        },
    })
}

/// Split a compound shell line into individual statements (respects quoted strings).
pub fn split_statements(line: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' && !in_double {
            in_single = !in_single;
            current.push(c);
            i += 1;
            continue;
        }
        if c == '"' && !in_single {
            in_double = !in_double;
            current.push(c);
            i += 1;
            continue;
        }
        if !in_single && !in_double {
            if c == '&' && i + 1 < chars.len() && chars[i + 1] == '&' {
                push_trimmed(&mut statements, &current);
                current.clear();
                i += 2;
                continue;
            }
            if c == '|' && i + 1 < chars.len() && chars[i + 1] == '|' {
                push_trimmed(&mut statements, &current);
                current.clear();
                i += 2;
                continue;
            }
            if c == ';' {
                push_trimmed(&mut statements, &current);
                current.clear();
                i += 1;
                continue;
            }
        }
        current.push(c);
        i += 1;
    }
    push_trimmed(&mut statements, &current);
    statements
}

fn push_trimmed(statements: &mut Vec<String>, raw: &str) {
    let trimmed = raw.trim();
    if !trimmed.is_empty() {
        statements.push(trimmed.to_string());
    }
}

/// Rewrite a compound line for a target shell.
pub fn translate_line(line: &str, target: ShellTarget) -> String {
    let parts = split_statements(line);
    if parts.len() <= 1 {
        return line.trim().to_string();
    }
    let sep = match target {
        ShellTarget::Posix | ShellTarget::Cmd | ShellTarget::PowerShell7 => " && ",
        ShellTarget::PowerShell5 => "; ",
    };
    parts.join(sep)
}

/// If the line is `cd PATH && playhouse ...`, return a playhouse -C rewrite.
pub fn rewrite_cd_playhouse(line: &str, workspace_hint: &str) -> Option<String> {
    let parts = split_statements(line);
    if parts.len() < 2 {
        return None;
    }
    let first = parts[0].trim();
    let rest = parts[1..].join(" && ");
    let path = first
        .strip_prefix("cd ")
        .or_else(|| first.strip_prefix("Set-Location "))
        .or_else(|| first.strip_prefix("set-location "))
        .map(str::trim)
        .map(|p| p.trim_matches('"').trim_matches('\''))?;
    if !rest.trim_start().starts_with("playhouse") {
        return None;
    }
    let ws = if path.is_empty() { workspace_hint } else { path };
    let cmd = rest.replacen("playhouse", &format!("playhouse -C \"{ws}\""), 1);
    Some(cmd)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ShellTarget {
    Posix,
    Cmd,
    PowerShell5,
    PowerShell7,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_and_and_semicolon() {
        let parts = split_statements(r#"cd "foo" && playhouse doctor --json"#);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], r#"cd "foo""#);
        assert_eq!(parts[1], "playhouse doctor --json");
    }

    #[test]
    fn rewrite_cd_playhouse_line() {
        let line = r#"cd "C:\proj" && playhouse agent status --json"#;
        let rewritten = rewrite_cd_playhouse(line, ".").unwrap();
        assert!(rewritten.contains(r#"playhouse -C "C:\proj""#));
        assert!(rewritten.contains("agent status --json"));
    }

    #[test]
    fn translate_powershell5_uses_semicolon() {
        let line = "cd foo && playhouse doctor --json";
        let out = translate_line(line, ShellTarget::PowerShell5);
        assert!(out.contains(';'));
        assert!(!out.contains("&&"));
    }
}
