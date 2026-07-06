use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::time::Duration;

use serde_json::Value;

use crate::config::load_settings;
use crate::pkgmgr::{detect_from_lockfiles, PackageManager};
use crate::project::{self, FunctionalRunner};
use crate::tools;
use crate::types::HealthCheck;

const FALLBACK_PORTS: [u16; 8] = [5173, 3000, 8080, 4200, 8000, 5000, 4321, 1313];

/// Ports inferred from package.json scripts and framework config files.
pub fn port_hints(workspace: &str) -> Vec<u16> {
    let scan = crate::workspace::scan_root_str(workspace);
    let root = Path::new(&scan);
    let mut ports = Vec::new();

    if let Ok(content) = fs::read_to_string(root.join("package.json")) {
        if let Ok(pkg) = serde_json::from_str::<Value>(&content) {
            if let Some(scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
                for (_, script) in scripts {
                    if let Some(s) = script.as_str() {
                        extract_ports_from_str(s, &mut ports);
                        framework_default_ports(s, &mut ports);
                    }
                }
            }
        }
    }

    for name in [
        "vite.config.ts",
        "vite.config.js",
        "vite.config.mjs",
        "astro.config.mjs",
        "astro.config.ts",
    ] {
        if let Ok(content) = fs::read_to_string(root.join(name)) {
            extract_keyed_port(&content, "port", &mut ports);
        }
    }

    if let Ok(content) = fs::read_to_string(root.join("wrangler.toml")) {
        extract_toml_port(&content, &mut ports);
    }

    ports.sort();
    ports.dedup();
    ports
}

/// First hinted port as a URL (no TCP probe).
pub fn suggested_local_url(workspace: &str) -> Option<String> {
    port_hints(workspace)
        .first()
        .map(|port| format!("http://localhost:{port}"))
}

/// Probe localhost for a responding dev server, preferring stack-specific port hints.
pub fn find_local_server(workspace: &str) -> Option<String> {
    let mut ports = port_hints(workspace);
    for port in FALLBACK_PORTS {
        if !ports.contains(&port) {
            ports.push(port);
        }
    }
    probe_ports(&ports)
}

fn probe_ports(ports: &[u16]) -> Option<String> {
    let timeout = Duration::from_millis(400);
    for port in ports {
        let addr: SocketAddr = format!("127.0.0.1:{port}").parse().ok()?;
        if TcpStream::connect_timeout(&addr, timeout).is_ok() {
            return Some(format!("http://localhost:{port}"));
        }
    }
    None
}

/// TCP probe for a configured http(s) URL (used before browser audits).
pub fn probe_url(url: &str) -> bool {
    let url = url.trim();
    let (scheme, rest) = if let Some(r) = url.strip_prefix("https://") {
        ("https", r)
    } else if let Some(r) = url.strip_prefix("http://") {
        ("http", r)
    } else {
        return false;
    };
    let host_port = rest.split('/').next().unwrap_or(rest);
    let (host, port) = if let Some((h, p)) = host_port.rsplit_once(':') {
        (h, p.parse::<u16>().ok())
    } else {
        (host_port, None)
    };
    let port = port.unwrap_or(if scheme == "https" { 443 } else { 80 });
    let host = match host {
        "localhost" | "127.0.0.1" => "127.0.0.1",
        other => other,
    };
    let Ok(addr) = format!("{host}:{port}").parse::<SocketAddr>() else {
        return false;
    };
    TcpStream::connect_timeout(&addr, Duration::from_millis(2000)).is_ok()
}

fn push_port(port: u16, ports: &mut Vec<u16>) {
    if (1..=65535).contains(&port) && !ports.contains(&port) {
        ports.push(port);
    }
}

fn extract_ports_from_str(text: &str, ports: &mut Vec<u16>) {
    for token in text.split_whitespace() {
        if let Some(rest) = token.strip_prefix("localhost:") {
            if let Ok(port) = rest.trim_end_matches('/').parse::<u16>() {
                push_port(port, ports);
            }
        }
        if let Some(rest) = token.strip_prefix("127.0.0.1:") {
            if let Ok(port) = rest.trim_end_matches('/').parse::<u16>() {
                push_port(port, ports);
            }
        }
        if let Some(rest) = token.strip_prefix("--port=") {
            if let Ok(port) = rest.parse::<u16>() {
                push_port(port, ports);
            }
        }
        if token == "--port" || token == "-p" {
            continue;
        }
        if let Some(rest) = token.strip_prefix("PORT=") {
            if let Ok(port) = rest.parse::<u16>() {
                push_port(port, ports);
            }
        }
    }

    let lower = text.to_lowercase();
    for needle in ["--port ", "-p "] {
        if let Some(idx) = lower.find(needle) {
            let rest = text[idx + needle.len()..].split_whitespace().next().unwrap_or("");
            if let Ok(port) = rest.parse::<u16>() {
                push_port(port, ports);
            }
        }
    }
}

fn framework_default_ports(script: &str, ports: &mut Vec<u16>) {
    let s = script.to_lowercase();
    if s.contains("vite") {
        push_port(5173, ports);
    }
    if s.contains("next") && (s.contains("dev") || s.contains("start")) {
        push_port(3000, ports);
    }
    if s.contains("wrangler") {
        push_port(8787, ports);
    }
    if s.contains("astro") && s.contains("dev") {
        push_port(4321, ports);
    }
    if s.contains("ng serve") {
        push_port(4200, ports);
    }
    if s.contains("react-scripts") || s.contains("craco") {
        push_port(3000, ports);
    }
}

fn extract_keyed_port(content: &str, key: &str, ports: &mut Vec<u16>) {
    let needle = format!("{key}:");
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.contains(&needle) {
            continue;
        }
        if let Some(idx) = trimmed.find(&needle) {
            let rest = &trimmed[idx + needle.len()..];
            let digits: String = rest
                .trim()
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(port) = digits.parse::<u16>() {
                push_port(port, ports);
            }
        }
    }
}

fn extract_toml_port(content: &str, ports: &mut Vec<u16>) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some((_, rhs)) = trimmed.split_once('=') {
            let rhs = rhs.trim().trim_matches('"').trim_matches('\'');
            if rhs.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(port) = rhs.parse::<u16>() {
                    push_port(port, ports);
                }
            }
        }
    }
}

fn command_version(cmd: &str, args: &[&str]) -> Option<String> {
    let out = crate::cmd::sync(cmd).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let line = text.lines().next()?.trim().to_string();
    if line.is_empty() { None } else { Some(line) }
}

pub fn check_playhouse_install() -> HealthCheck {
    if let Ok(method) = std::env::var("PLAYHOUSE_INSTALL_METHOD") {
        let detail = match method.as_str() {
            "npm" => {
                let root = std::env::var("PLAYHOUSE_NPM_ROOT")
                    .unwrap_or_else(|_| "node_modules/playhouse".into());
                let version = env!("CARGO_PKG_VERSION");
                format!("npm-bundled v{version} ({root})")
            }
            "PLAYHOUSE_BIN" => {
                let bin = std::env::var("PLAYHOUSE_BIN").unwrap_or_else(|_| "custom".into());
                format!("PLAYHOUSE_BIN ({bin})")
            }
            other => other.to_string(),
        };
        return HealthCheck::pass("Playhouse CLI", &detail);
    }

    let exe = std::env::current_exe().ok();
    let exe_display = exe
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "playhouse".into());
    let version = env!("CARGO_PKG_VERSION");
    let detail = match classify_install_source(&exe_display) {
        "npm-bundled" => format!("npm-bundled v{version}"),
        "cargo-local" => format!("cargo local build v{version} ({exe_display})"),
        "cargo-install" => format!("cargo install v{version}"),
        _ => format!("PATH v{version} ({exe_display})"),
    };

    HealthCheck::pass("Playhouse CLI", &detail)
}

fn classify_install_source(exe_display: &str) -> &'static str {
    let from_npm = exe_display.contains("node_modules") && exe_display.contains("playhouse");
    let from_cargo_build =
        exe_display.contains("\\target\\") || exe_display.contains("/target/");
    let from_cargo_install = exe_display.contains(".cargo\\bin")
        || exe_display.contains(".cargo/bin")
        || exe_display.contains("\\cargo\\bin");

    if from_npm {
        "npm-bundled"
    } else if from_cargo_build {
        "cargo-local"
    } else if from_cargo_install {
        "cargo-install"
    } else {
        "path"
    }
}

fn command_ok(cmd: &str, args: &[&str]) -> bool {
    crate::cmd::sync(cmd)
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn check_node() -> HealthCheck {
    match command_version("node", &["--version"]) {
        Some(v) => HealthCheck::pass("Node.js", &v),
        None => HealthCheck::warn("Node.js", "not found (required for Playwright and Lighthouse)"),
    }
}

pub fn check_npm() -> HealthCheck {
    match command_version(crate::cmd::npm_program(), &["--version"]) {
        Some(v) => HealthCheck::pass("npm", &v),
        None => HealthCheck::warn("npm", "not found"),
    }
}

pub fn check_pnpm() -> HealthCheck {
    let cmd = if cfg!(windows) { "pnpm.cmd" } else { "pnpm" };
    match command_version(cmd, &["--version"]) {
        Some(v) => HealthCheck::pass("pnpm", &v),
        None => HealthCheck::warn("pnpm", "not found"),
    }
}

pub fn check_yarn() -> HealthCheck {
    let cmd = if cfg!(windows) { "yarn.cmd" } else { "yarn" };
    match command_version(cmd, &["--version"]) {
        Some(v) => HealthCheck::pass("yarn", &v),
        None => HealthCheck::warn("yarn", "not found"),
    }
}

pub fn check_bun() -> HealthCheck {
    let cmd = if cfg!(windows) { "bun.exe" } else { "bun" };
    match command_version(cmd, &["--version"]) {
        Some(v) => HealthCheck::pass("bun", &v),
        None => HealthCheck::warn("bun", "not found"),
    }
}

pub fn check_package_manager(workspace: &str) -> HealthCheck {
    let settings = load_settings();
    let pm = PackageManager::resolve(workspace, &settings.package_manager);
    let detected = detect_from_lockfiles(workspace)
        .map(|p| p.label().to_string())
        .unwrap_or_else(|| "none".into());
    let detail = if settings.package_manager == "auto" {
        format!("using {} (lockfile: {detected})", pm.label())
    } else {
        format!("configured: {} -> {}", settings.package_manager, pm.label())
    };
    if pm.is_available() {
        HealthCheck::pass("Package manager", &detail)
    } else {
        HealthCheck::fail("Package manager", &format!("{detail} - not available on PATH"))
    }
}

pub fn check_playwright_test(workspace: &str) -> HealthCheck {
    if let Some(bin) = tools::project_node_bin(workspace, "playwright") {
        return HealthCheck::pass(
            "@playwright/test",
            &format!("project node_modules: {}", bin.display()),
        );
    }
    if !tools::has_playwright(workspace) {
        return HealthCheck::warn("@playwright/test", "not installed - run: playhouse install --full");
    }
    let settings = load_settings();
    let pm = PackageManager::resolve(workspace, &settings.package_manager);
    let ctx = tools::resolve_node_tool_context(workspace);
    let program = pm.program();
    let args: Vec<&str> = match pm {
        PackageManager::Npm => vec!["exec", "--", "playwright", "--version"],
        PackageManager::Pnpm => vec!["exec", "playwright", "--version"],
        PackageManager::Yarn => vec!["playwright", "--version"],
        PackageManager::Bun => vec!["x", "playwright", "--version"],
    };
    let out = crate::cmd::sync(program)
        .args(&args)
        .current_dir(&ctx.cwd)
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
            HealthCheck::pass(
                "@playwright/test",
                &format!(".playhouse/npm via {}: {v}", pm.label()),
            )
        }
        _ => HealthCheck::warn("@playwright/test", "installed but version check failed"),
    }
}

pub fn check_lighthouse(workspace: &str) -> HealthCheck {
    if let Some(bin) = tools::project_node_bin(workspace, "lighthouse") {
        return HealthCheck::pass("Lighthouse", &format!("project: {}", bin.display()));
    }
    if tools::bundled_node_bin(workspace, "lighthouse").is_some() {
        return HealthCheck::pass("Lighthouse", "bundled in .playhouse/npm");
    }
    if command_ok("lighthouse", &["--version"]) {
        match command_version("lighthouse", &["--version"]) {
            Some(v) => HealthCheck::pass("Lighthouse", &format!("global: {v}")),
            None => HealthCheck::pass("Lighthouse", "global: installed"),
        }
    } else {
        HealthCheck::warn("Lighthouse", "not installed - run: playhouse install --full")
    }
}

pub fn check_trivy() -> HealthCheck {
    let program = tools::trivy_program();
    match command_version(&program, &["--version"]) {
        Some(v) => {
            let short = v.lines().next().unwrap_or(&v).to_string();
            let loc = if tools::has_bundled_trivy() {
                "bundled"
            } else {
                "PATH"
            };
            HealthCheck::pass("Trivy", &format!("{loc}: {short}"))
        }
        None => HealthCheck::warn("Trivy", "not installed - run: playhouse install"),
    }
}

pub fn check_local_server(workspace: &str) -> HealthCheck {
    let hints = port_hints(workspace);
    match find_local_server(workspace) {
        Some(url) => {
            let hint_note = if hints.is_empty() {
                String::new()
            } else {
                format!(" (hints: {})", hints.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "))
            };
            HealthCheck::pass(
                "Local server",
                &format!("responding at {url}{hint_note}"),
            )
        }
        None => {
            let detail = if hints.is_empty() {
                "no dev server detected on common ports".into()
            } else {
                format!(
                    "no server on hinted ports ({}); start dev server or: playhouse config set default_url http://localhost:{}",
                    hints.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "),
                    hints[0]
                )
            };
            HealthCheck::warn("Local server", &detail)
        }
    }
}

pub fn check_workspace_deps(workspace: &str, package_manager: &str) -> HealthCheck {
    let scan = crate::workspace::scan_root(workspace);
    let is_node_project = scan.join("package.json").is_file();
    if !is_node_project {
        return HealthCheck::pass("Workspace dependencies", "n/a (no package.json)");
    }

    let node_modules = scan.join("node_modules");
    let has_installed_deps = node_modules.is_dir()
        && std::fs::read_dir(&node_modules)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false);
    if has_installed_deps {
        return HealthCheck::pass("Workspace dependencies", "node_modules present");
    }

    let pm = PackageManager::resolve(workspace, package_manager);
    let hint = match pm {
        PackageManager::Npm => "npm install",
        PackageManager::Pnpm => "pnpm install",
        PackageManager::Yarn => "yarn install",
        PackageManager::Bun => "bun install",
    };
    HealthCheck::warn(
        "Workspace dependencies",
        &format!(
            "node_modules missing or empty - run {hint}{}",
            windows_install_lock_note()
        ),
    )
}

#[cfg(windows)]
fn windows_install_lock_note() -> &'static str {
    "; if install fails with EPERM/EBUSY, close IDE/antivirus locks and run playhouse install --json"
}

#[cfg(not(windows))]
fn windows_install_lock_note() -> &'static str {
    ""
}

const NATIVE_PROBE_DEPS: &[&str] = &[
    "sqlite3",
    "better-sqlite3",
    "bcrypt",
    "sharp",
    "canvas",
    "node-sass",
    "argon2",
];

pub fn check_native_bindings(workspace: &str) -> HealthCheck {
    let scan = crate::workspace::scan_root(workspace);
    if !scan.join("package.json").is_file() {
        return HealthCheck::pass("Native bindings", "n/a (no package.json)");
    }
    let node_modules = scan.join("node_modules");
    if !node_modules.is_dir() {
        return HealthCheck::warn(
            "Native bindings",
            "node_modules missing — run install before probing native modules",
        );
    }

    let deps = native_deps_in_project(&scan);
    if deps.is_empty() {
        return HealthCheck::pass("Native bindings", "no known native deps declared");
    }

    let dep_count = deps.len();
    let mut failures = Vec::new();
    for dep in &deps {
        if let Some(err) = probe_native_module(&scan, dep) {
            failures.push(format!("{dep}: {err}"));
        }
    }

    if failures.is_empty() {
        HealthCheck::pass("Native bindings", &format!("{dep_count} module(s) load OK"))
    } else {
        HealthCheck::warn(
            "Native bindings",
            &format!(
                "{} — try: playhouse doctor --resolve",
                failures.join("; ")
            ),
        )
    }
}

fn native_deps_in_project(scan: &std::path::Path) -> Vec<String> {
    let content = match std::fs::read_to_string(scan.join("package.json")) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let pkg: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let mut found = Vec::new();
    for key in ["dependencies", "devDependencies", "optionalDependencies"] {
        if let Some(obj) = pkg.get(key).and_then(|v| v.as_object()) {
            for dep in NATIVE_PROBE_DEPS {
                if obj.contains_key(*dep) {
                    found.push(dep.to_string());
                }
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

fn probe_native_module(scan: &std::path::Path, dep: &str) -> Option<String> {
    if !scan.join("node_modules").join(dep).is_dir() {
        return None;
    }
    let script = format!("try{{require('{dep}');}}catch(e){{console.error(e.message);process.exit(1)}}");
    let out = crate::cmd::sync("node")
        .args(["-e", &script])
        .current_dir(scan)
        .output()
        .ok()?;
    if out.status.success() {
        None
    } else {
        Some(
            String::from_utf8_lossy(&out.stderr)
                .lines()
                .next()
                .unwrap_or("load failed")
                .chars()
                .take(120)
                .collect(),
        )
    }
}

pub fn check_arkenar() -> HealthCheck {
    let program = tools::arkenar_program();
    if crate::cmd::sync(&program)
        .arg("--help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        let loc = if tools::has_bundled_arkenar() {
            "bundled"
        } else {
            "PATH"
        };
        HealthCheck::pass("Arkenar DAST", &format!("{loc} - MIT Rust scanner"))
    } else {
        HealthCheck::warn("Arkenar DAST", "not installed - run: playhouse install")
    }
}

pub fn check_cargo() -> HealthCheck {
    match command_version("cargo", &["--version"]) {
        Some(v) => HealthCheck::pass("cargo", &v),
        None => HealthCheck::warn("cargo", "not found - install Rust: https://rustup.rs"),
    }
}

pub fn check_go() -> HealthCheck {
    match command_version("go", &["version"]) {
        Some(v) => HealthCheck::pass("go", &v),
        None => HealthCheck::warn("go", "not found - install Go: https://go.dev/dl/"),
    }
}

pub fn check_pytest() -> HealthCheck {
    let program = if cfg!(windows) { "pytest.exe" } else { "pytest" };
    if let Some(v) = command_version(program, &["--version"]) {
        return HealthCheck::pass("pytest", &v);
    }
    if let Some(v) = command_version(
        if cfg!(windows) { "python.exe" } else { "python" },
        &["-m", "pytest", "--version"],
    ) {
        return HealthCheck::pass("pytest", &format!("python -m pytest: {v}"));
    }
    HealthCheck::warn("pytest", "not found - pip install pytest")
}

pub fn check_java_runner(workspace: &str, runner: FunctionalRunner) -> HealthCheck {
    let root = std::path::Path::new(workspace);
    match runner {
        FunctionalRunner::MvnTest => match command_version("mvn", &["--version"]) {
            Some(v) => {
                let line = v.lines().next().unwrap_or(&v).to_string();
                HealthCheck::pass("Maven", &line)
            }
            None => HealthCheck::warn("Maven", "not found - required for mvn test"),
        },
        FunctionalRunner::GradleTest => {
            let gradle = if root.join("gradlew.bat").is_file() {
                root.join("gradlew.bat").to_string_lossy().into_owned()
            } else if root.join("gradlew").is_file() {
                root.join("gradlew").to_string_lossy().into_owned()
            } else {
                "gradle".into()
            };
            match command_version(&gradle, &["--version"]) {
                Some(v) => {
                    let line = v.lines().next().unwrap_or(&v).to_string();
                    HealthCheck::pass("Gradle", &line)
                }
                None => HealthCheck::warn("Gradle", "not found - required for gradle test"),
            }
        }
        _ => HealthCheck::pass("Java toolchain", "n/a"),
    }
}

pub fn run_doctor(workspace: &str) -> Vec<HealthCheck> {
    let profile = project::detect(workspace);
    let settings = load_settings();

    let mut checks = vec![
        check_playhouse_install(),
        check_trivy(),
        check_arkenar(),
    ];

    if profile.needs_node() {
        checks.push(check_node());
        checks.push(check_package_manager(workspace));
        checks.push(check_npm());
        checks.push(check_workspace_deps(workspace, &settings.package_manager));
        checks.push(check_native_bindings(workspace));
        if project::needs_alt_package_manager_checks(workspace, &settings.package_manager) {
            checks.push(check_pnpm());
            checks.push(check_yarn());
            checks.push(check_bun());
        }
    }

    match profile.functional_runner {
        FunctionalRunner::Playwright => {
            checks.push(check_playwright_test(workspace));
        }
        FunctionalRunner::CargoTest => {
            checks.push(check_cargo());
        }
        FunctionalRunner::GoTest => {
            checks.push(check_go());
        }
        FunctionalRunner::Pytest => {
            checks.push(check_pytest());
        }
        FunctionalRunner::NpmTest => {}
        FunctionalRunner::MvnTest | FunctionalRunner::GradleTest => {
            checks.push(check_java_runner(workspace, profile.functional_runner));
        }
        FunctionalRunner::None => {}
    }

    if profile.browser_audits {
        checks.push(check_lighthouse(workspace));
        checks.push(check_local_server(workspace));
    }

    checks
}

/// Rebuild native Node bindings in the project scan root (`npm rebuild`, etc.).
pub async fn resolve_native_bindings(workspace: &str) -> Option<String> {
    let scan = crate::workspace::scan_root(workspace);
    if !scan.join("package.json").is_file() {
        return None;
    }
    let settings = crate::config::load_settings();
    let pm = crate::pkgmgr::PackageManager::resolve(workspace, &settings.package_manager);
    let out = crate::cmd::r#async(pm.program())
        .arg("rebuild")
        .current_dir(&scan)
        .output()
        .await
        .ok()?;
    if out.status.success() {
        Some(format!("{} rebuild completed", pm.label()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn port_hints_from_script_flags() {
        let dir = std::env::temp_dir().join(format!("playhouse-hints-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("package.json"),
            r#"{"scripts":{"dev":"vite --port 4173","start":"next dev -p 3001"}}"#,
        )
        .unwrap();
        let hints = port_hints(dir.to_str().unwrap());
        assert!(hints.contains(&4173));
        assert!(hints.contains(&3001));
        assert!(hints.contains(&5173)); // vite default
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn port_hints_from_vite_config() {
        let dir = std::env::temp_dir().join(format!("playhouse-vite-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("vite.config.ts"),
            "export default { server: { port: 5199 } }",
        )
        .unwrap();
        let hints = port_hints(dir.to_str().unwrap());
        assert!(hints.contains(&5199));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn suggested_local_url_uses_first_hint() {
        let dir = std::env::temp_dir().join(format!("playhouse-url-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("package.json"),
            r#"{"scripts":{"dev":"vite --port 4173"}}"#,
        )
        .unwrap();
        assert_eq!(
            suggested_local_url(dir.to_str().unwrap()).as_deref(),
            Some("http://localhost:4173")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn port_hints_wrangler_toml() {
        let dir = std::env::temp_dir().join(format!("playhouse-wrangler-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("wrangler.toml"), "port = 8787\n").unwrap();
        let hints = port_hints(dir.to_str().unwrap());
        assert!(hints.contains(&8787));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_playhouse_install_reports_version() {
        let check = check_playhouse_install();
        assert_eq!(check.name, "Playhouse CLI");
        assert!(check.detail.contains(env!("CARGO_PKG_VERSION")));
    }
}
