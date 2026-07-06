use std::path::Path;

pub const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".playhouse",
    ".cursor",
    "dist",
    "build",
    "__pycache__",
    ".next",
    "vendor",
];

pub const MAX_DEPTH: usize = 8;
pub const MAX_FILES: usize = 3_000;

pub fn should_skip_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name) || (name.starts_with('.') && name != "." && name != "..")
}

pub fn collect_files(workspace: &Path) -> Vec<String> {
    let mut files = Vec::with_capacity(512);
    if workspace.is_dir() {
        collect_files_inner(workspace, workspace, &mut files, 0);
    }
    files.sort();
    files.truncate(MAX_FILES);
    files
}

fn collect_files_inner(base: &Path, dir: &Path, files: &mut Vec<String>, depth: usize) {
    if depth > MAX_DEPTH || files.len() >= MAX_FILES {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        if files.len() >= MAX_FILES {
            return;
        }

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        if path.is_dir() {
            if !should_skip_dir(&name) {
                collect_files_inner(base, &path, files, depth + 1);
            }
            continue;
        }

        if !path.is_file() {
            continue;
        }

        if let Ok(rel) = path.strip_prefix(base) {
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if !rel_str.is_empty() {
                files.push(rel_str);
            }
        }
    }
}
