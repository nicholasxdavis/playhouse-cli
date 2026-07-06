use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::cmd::sync;
use crate::config::load_settings;
use crate::pkgmgr::PackageManager;
use crate::tools::{
    self, arkenar_binary_name, bin_dir, playwright_npm_dir, trivy_binary_name, ARKENAR_VERSION,
    LIGHTHOUSE_PKG, PLAYWRIGHT_PKG, TRIVY_VERSION,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallProfile {
    Minimal,
    Full,
}

impl InstallProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Full => "full",
        }
    }

    pub fn from_flags(minimal: bool, _full: bool) -> Self {
        if minimal {
            Self::Minimal
        } else {
            Self::Full
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallIssue {
    pub component: String,
    pub message: String,
    pub error_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct InstallReport {
    pub profile: String,
    pub trivy: bool,
    pub arkenar: bool,
    pub playwright: bool,
    pub lighthouse: bool,
    pub messages: Vec<String>,
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<InstallIssue>,
}

impl InstallReport {
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

pub async fn ensure_all(workspace: &str, quiet: bool) -> InstallReport {
    ensure_profile(workspace, InstallProfile::Full, quiet).await
}

pub async fn ensure_profile(
    workspace: &str,
    profile: InstallProfile,
    quiet: bool,
) -> InstallReport {
    let mut report = InstallReport {
        profile: profile.as_str().into(),
        ..Default::default()
    };

    match ensure_trivy(quiet).await {
        Ok(msg) => {
            report.trivy = true;
            report.messages.push(msg);
        }
        Err(e) => record_install_error(&mut report, "trivy", &e),
    }

    match ensure_arkenar(quiet).await {
        Ok(msg) => {
            report.arkenar = true;
            report.messages.push(msg);
        }
        Err(e) => record_install_error(&mut report, "arkenar", &e),
    }

    if profile == InstallProfile::Full {
        match ensure_web_tools(workspace, quiet).await {
            Ok(msgs) => {
                report.playwright = true;
                report.lighthouse = true;
                report.messages.extend(msgs);
            }
            Err(e) => {
                record_install_error(&mut report, "web-tools", &e);
            }
        }
    }

    report
}

fn record_install_error(report: &mut InstallReport, component: &str, err: &str) {
    let kind = crate::pkgmgr::classify_install_error(err);
    report
        .errors
        .push(sanitize_error(component, err));
    report.issues.push(InstallIssue {
        component: component.into(),
        message: err.lines().next().unwrap_or(err).trim().chars().take(500).collect(),
        error_kind: kind.as_str().into(),
        remediation: kind.remediation().map(String::from),
    });
}

fn sanitize_error(tool: &str, err: &str) -> String {
    let one_line = err.lines().next().unwrap_or(err).trim();
    let truncated: String = one_line.chars().take(200).collect();
    format!("{tool}: {truncated}")
}

pub async fn ensure_trivy(quiet: bool) -> Result<String, String> {
    if tools::has_bundled_trivy() {
        return Ok("Trivy already installed (bundled)".into());
    }

    if sync(&tools::trivy_program())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Ok("Trivy available on PATH".into());
    }

    install_trivy(quiet).await
}

pub async fn install_trivy(quiet: bool) -> Result<String, String> {
    let _ = fs::create_dir_all(bin_dir());

    let (url, archive_name, archive_kind) = trivy_download_spec()?;
    let cache_dir = playhouse_home_cache();
    let _ = fs::create_dir_all(&cache_dir);
    let archive_path = cache_dir.join(&archive_name);

    if !quiet {
        eprintln!("[*] Downloading Trivy {TRIVY_VERSION}...");
    }
    download_file(&url, &archive_path)?;

    let extract_dir = cache_dir.join("trivy-extract");
    let _ = fs::remove_dir_all(&extract_dir);
    fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;
    extract_archive(&archive_path, &extract_dir, archive_kind)?;

    let dest = tools::bundled_trivy_path();
    let found = find_file_named(&extract_dir, trivy_binary_name())
        .ok_or_else(|| "trivy binary not found in archive".to_string())?;
    fs::copy(&found, &dest).map_err(|e| e.to_string())?;
    set_executable(&dest)?;

    Ok(format!("Installed Trivy to {}", dest.display()))
}

pub async fn ensure_playwright(workspace: &str, quiet: bool) -> Result<String, String> {
    if tools::project_node_bin(workspace, "playwright").is_some() {
        return Ok("Playwright available in project node_modules".into());
    }
    if tools::bundled_node_bin(workspace, "playwright").is_some() {
        return Ok("Playwright already installed in .playhouse/npm".into());
    }
    if !node_available() {
        return Err("Node.js is required to install Playwright (https://nodejs.org)".into());
    }
    install_web_tools(workspace, quiet).await.map(|_| {
        "Installed Playwright in .playhouse/npm".into()
    })
}

pub async fn ensure_lighthouse(workspace: &str, quiet: bool) -> Result<String, String> {
    if tools::has_lighthouse(workspace) {
        let src = if tools::project_node_bin(workspace, "lighthouse").is_some() {
            "project node_modules"
        } else {
            ".playhouse/npm"
        };
        return Ok(format!("Lighthouse available ({src})"));
    }
    if !node_available() {
        return Err("Node.js is required to install Lighthouse (https://nodejs.org)".into());
    }
    install_web_tools(workspace, quiet).await.map(|_| {
        "Installed Lighthouse in .playhouse/npm".into()
    })
}

/// Install @playwright/test + lighthouse + chromium into `.playhouse/npm/`.
pub async fn ensure_web_tools(workspace: &str, quiet: bool) -> Result<Vec<String>, String> {
    if tools::project_node_bin(workspace, "playwright").is_some()
        && tools::has_lighthouse(workspace)
    {
        return Ok(vec![
            "Playwright: project node_modules".into(),
            "Lighthouse: available".into(),
        ]);
    }

    install_web_tools(workspace, quiet).await
}

pub async fn install_web_tools(workspace: &str, quiet: bool) -> Result<Vec<String>, String> {
    let settings = load_settings();
    let pm = PackageManager::resolve(workspace, &settings.package_manager);
    let npm_dir = playwright_npm_dir(workspace);
    fs::create_dir_all(&npm_dir).map_err(|e| e.to_string())?;

    ensure_tools_package_json(&npm_dir)?;

    if !quiet {
        eprintln!(
            "[*] Installing @playwright/test + lighthouse via {} into .playhouse/npm...",
            pm.label()
        );
    }

    pm.install_resilient(&npm_dir)
        .await
        .map_err(|e| format!("{} install failed: {e}", pm.label()))?;

    if tools::bundled_node_bin(workspace, "playwright").is_none() {
        return Err("@playwright/test missing after install".into());
    }
    if tools::bundled_node_bin(workspace, "lighthouse").is_none() {
        return Err("lighthouse missing after install".into());
    }

    if !quiet {
        eprintln!("[*] Installing Playwright browser (chromium)...");
    }

    pm.install_playwright_browser(&npm_dir)
        .await
        .map_err(|e| format!("playwright install failed: {e}"))?;

    Ok(vec![
        format!("Playwright via {} in {}", pm.label(), npm_dir.display()),
        format!("Lighthouse via {} in {}", pm.label(), npm_dir.display()),
    ])
}

fn ensure_tools_package_json(npm_dir: &Path) -> Result<(), String> {
    let package_json = npm_dir.join("package.json");
    let mut pkg: serde_json::Value = if package_json.is_file() {
        let content = fs::read_to_string(&package_json).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or_else(|_| empty_tools_package())
    } else {
        empty_tools_package()
    };

    let deps = pkg
        .as_object_mut()
        .and_then(|o| o.get_mut("devDependencies"))
        .and_then(|d| d.as_object_mut())
        .ok_or_else(|| "invalid package.json shape".to_string())?;

    deps.insert(
        "@playwright/test".into(),
        serde_json::Value::String(PLAYWRIGHT_PKG.into()),
    );
    deps.insert(
        "lighthouse".into(),
        serde_json::Value::String(LIGHTHOUSE_PKG.into()),
    );

    fs::write(
        &package_json,
        serde_json::to_string_pretty(&pkg).unwrap_or_default(),
    )
    .map_err(|e| e.to_string())
}

fn empty_tools_package() -> serde_json::Value {
    serde_json::json!({
        "name": "playhouse-tools",
        "private": true,
        "devDependencies": {}
    })
}

pub async fn ensure_arkenar(quiet: bool) -> Result<String, String> {
    if tools::has_bundled_arkenar() {
        return Ok("Arkenar already installed (bundled)".into());
    }

    if sync(&tools::arkenar_program())
        .arg("--help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Ok("Arkenar available on PATH".into());
    }

    install_arkenar(quiet).await
}

pub async fn install_arkenar(quiet: bool) -> Result<String, String> {
    let _ = fs::create_dir_all(bin_dir());

    let (url, archive_name, archive_kind) = arkenar_download_spec()?;
    let cache_dir = playhouse_home_cache();
    let _ = fs::create_dir_all(&cache_dir);
    let archive_path = cache_dir.join(&archive_name);

    if !quiet {
        eprintln!("[*] Downloading Arkenar v{ARKENAR_VERSION}...");
    }
    download_file(&url, &archive_path)?;

    let extract_dir = cache_dir.join("arkenar-extract");
    let _ = fs::remove_dir_all(&extract_dir);
    fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;
    extract_archive(&archive_path, &extract_dir, archive_kind)?;

    let dest = tools::bundled_arkenar_path();
    let found = find_file_named(&extract_dir, arkenar_binary_name())
        .ok_or_else(|| "arkenar binary not found in archive".to_string())?;
    fs::copy(&found, &dest).map_err(|e| e.to_string())?;
    set_executable(&dest)?;

    Ok(format!("Installed Arkenar to {}", dest.display()))
}

fn arkenar_download_spec() -> Result<(String, String, ArchiveKind), String> {
    let base = format!(
        "https://github.com/realozk/ARKENAR/releases/download/v{ARKENAR_VERSION}"
    );
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        Ok((
            format!("{base}/arkenar-windows-amd64.zip"),
            "arkenar-windows-amd64.zip".to_string(),
            ArchiveKind::Zip,
        ))
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        Ok((
            format!("{base}/arkenar-linux-amd64.tar.gz"),
            "arkenar-linux-amd64.tar.gz".to_string(),
            ArchiveKind::TarGz,
        ))
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        Ok((
            format!("{base}/arkenar-linux-arm64.tar.gz"),
            format!("arkenar-linux-arm64.tar.gz"),
            ArchiveKind::TarGz,
        ))
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        Ok((
            format!("{base}/arkenar-macos-amd64.tar.gz"),
            format!("arkenar-macos-amd64.tar.gz"),
            ArchiveKind::TarGz,
        ))
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        Ok((
            format!("{base}/arkenar-macos-arm64.tar.gz"),
            format!("arkenar-macos-arm64.tar.gz"),
            ArchiveKind::TarGz,
        ))
    }
    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
    )))]
    {
        Err("Unsupported platform for bundled Arkenar install".into())
    }
}

fn node_available() -> bool {
    sync(if cfg!(windows) { "node.exe" } else { "node" })
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn playhouse_home_cache() -> std::path::PathBuf {
    crate::config::playhouse_home().join("cache")
}

#[derive(Clone, Copy)]
enum ArchiveKind {
    #[allow(dead_code)]
    Zip,
    #[allow(dead_code)]
    TarGz,
}

fn trivy_download_spec() -> Result<(String, String, ArchiveKind), String> {
    let base = format!(
        "https://github.com/aquasecurity/trivy/releases/download/v{TRIVY_VERSION}/trivy_{TRIVY_VERSION}"
    );
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        Ok((
            format!("{base}_windows-64bit.zip"),
            format!("trivy_{TRIVY_VERSION}_windows-64bit.zip"),
            ArchiveKind::Zip,
        ))
    }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        // Trivy 0.72.0 has no native Windows ARM64 build; x64 runs under WoA emulation.
        Ok((
            format!("{base}_windows-64bit.zip"),
            format!("trivy_{TRIVY_VERSION}_windows-64bit.zip"),
            ArchiveKind::Zip,
        ))
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        Ok((
            format!("{base}_Linux-64bit.tar.gz"),
            format!("trivy_{TRIVY_VERSION}_Linux-64bit.tar.gz"),
            ArchiveKind::TarGz,
        ))
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        Ok((
            format!("{base}_Linux-ARM64.tar.gz"),
            format!("trivy_{TRIVY_VERSION}_Linux-ARM64.tar.gz"),
            ArchiveKind::TarGz,
        ))
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        Ok((
            format!("{base}_macOS-64bit.tar.gz"),
            format!("trivy_{TRIVY_VERSION}_macOS-64bit.tar.gz"),
            ArchiveKind::TarGz,
        ))
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        Ok((
            format!("{base}_macOS-ARM64.tar.gz"),
            format!("trivy_{TRIVY_VERSION}_macOS-ARM64.tar.gz"),
            ArchiveKind::TarGz,
        ))
    }
    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "aarch64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
    )))]
    {
        Err("Unsupported platform for bundled Trivy install".into())
    }
}

fn set_executable(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).map_err(|e| e.to_string())?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        let script = format!(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
            url.replace('\'', "''"),
            dest.display().to_string().replace('\'', "''")
        );
        let out = sync("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let out = sync("curl")
            .args(["-fsSL", url, "-o", dest.to_str().unwrap_or("trivy.bin")])
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
        }
        Ok(())
    }
}

fn extract_archive(archive: &Path, dest: &Path, kind: ArchiveKind) -> Result<(), String> {
    match kind {
        ArchiveKind::Zip => {
            #[cfg(windows)]
            {
                let script = format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    archive.display().to_string().replace('\'', "''"),
                    dest.display().to_string().replace('\'', "''")
                );
                let out = sync("powershell")
                    .args(["-NoProfile", "-Command", &script])
                    .output()
                    .map_err(|e| e.to_string())?;
                if !out.status.success() {
                    return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
                }
            }
            #[cfg(not(windows))]
            {
                let out = sync("unzip")
                    .args(["-o", archive.to_str().unwrap_or(""), "-d", dest.to_str().unwrap_or("")])
                    .output()
                    .map_err(|e| e.to_string())?;
                if !out.status.success() {
                    return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
                }
            }
        }
        ArchiveKind::TarGz => {
            let out = sync("tar")
                .args(["-xzf", archive.to_str().unwrap_or(""), "-C", dest.to_str().unwrap_or("")])
                .output()
                .map_err(|e| e.to_string())?;
            if !out.status.success() {
                return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
            }
        }
    }
    Ok(())
}

fn find_file_named(dir: &Path, name: &str) -> Option<std::path::PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(found) = find_file_named(&path, name) {
                return Some(found);
            }
        }
    }
    None
}
