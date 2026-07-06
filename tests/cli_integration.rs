use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn playhouse_bin() -> &'static str {
    env!("CARGO_BIN_EXE_playhouse")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn run_ok(args: &[&str], cwd: &Path) -> String {
    let out = Command::new(playhouse_bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| panic!("failed to run playhouse {args:?}: {e}"));
    assert!(
        out.status.success(),
        "playhouse {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn cli_version() {
    let out = Command::new(playhouse_bin())
        .arg("--version")
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn agent_manifest_has_workspace_block() {
    let stdout = run_ok(&["agent", "--json"], &repo_root());
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(v.get("workspace").is_some());
    assert!(v.get("tests").is_some());
}

#[test]
fn config_schema_lists_monorepo_keys() {
    let stdout = run_ok(&["config", "schema", "--json"], &repo_root());
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let keys: Vec<String> = v["keys"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|k| k["key"].as_str().map(String::from))
        .collect();
    assert!(keys.contains(&"scan_root".to_string()));
    assert!(keys.contains(&"test_root".to_string()));
    assert!(keys.contains(&"functional_runner".to_string()));
}

#[test]
fn functional_runs_in_fixture_crate() {
    let fixture = repo_root().join("tests/fixtures/rust-app");
    let stdout = run_ok(&["functional", "--json"], &fixture);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["runner"], "cargo-test");
    assert_eq!(v["stats"]["passed"], 1);
}

#[test]
fn monorepo_scan_root_in_temp_workspace() {
    let root = repo_root();
    let temp = std::env::temp_dir().join(format!("playhouse-mono-it-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).unwrap();
    fs::create_dir_all(temp.join(".playhouse")).unwrap();
    fs::write(
        temp.join(".playhouse/config.json"),
        r#"{"scan_root":"pkg","test_root":"pkg"}"#,
    )
    .unwrap();
    fs::create_dir_all(temp.join("pkg")).unwrap();
    fs::copy(
        root.join("tests/fixtures/rust-app/Cargo.toml"),
        temp.join("pkg/Cargo.toml"),
    )
    .unwrap();
    fs::create_dir_all(temp.join("pkg/src")).unwrap();
    fs::copy(
        root.join("tests/fixtures/rust-app/src/lib.rs"),
        temp.join("pkg/src/lib.rs"),
    )
    .unwrap();

    let stdout = run_ok(&["functional", "--json"], &temp);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["runner"], "cargo-test");
    assert_eq!(v["stats"]["passed"], 1);

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn playwright_delegates_to_cargo_on_rust_fixture() {
    let fixture = repo_root().join("tests/fixtures/rust-app");
    let stdout = run_ok(&["playwright", "--json"], &fixture);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["runner"], "cargo-test");
    assert_eq!(v["stats"]["passed"], 1);
}

#[test]
fn upgrade_command_json() {
    let stdout = run_ok(&["upgrade", "--json"], &repo_root());
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["current"], env!("CARGO_PKG_VERSION"));
    assert!(v.get("github").is_some());
    assert!(v.get("npm").is_some());
}
