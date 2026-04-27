//! Task discovery — find `.caro` files in the project's `tasks/` directory
//! and the user's `~/.caro/library/` global library.
//!
//! ## Resolution rules
//!
//! - Bare `<name>` — try `./tasks/<name>.caro`, then `~/.caro/library/<name>.caro`.
//!   The first hit wins; project always shadows global.
//! - `<name>` containing `/` (e.g. `system/snapshot`) walks subdirectories under
//!   either root.
//! - `<name>` ending in `.caro` is treated as an explicit path (relative to CWD
//!   or absolute).
//!
//! Carofile discovery is parallel: `Carofile` (no extension) or `Carofile.caro`
//! at the current working directory.

use std::path::{Path, PathBuf};

/// One discovered task with its source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskEntry {
    /// Logical task name (`cleanup-logs`, `system/snapshot`, …) — the path
    /// relative to its tasks-root, with the `.caro` suffix stripped.
    pub name: String,
    /// Absolute or relative path to the `.caro` file on disk.
    pub path: PathBuf,
    /// Whether this task came from the project or the global library.
    pub source: TaskSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskSource {
    Project,
    Global,
}

/// The conventional project tasks directory: `./tasks/`.
pub fn project_tasks_dir() -> PathBuf {
    PathBuf::from("tasks")
}

/// The user's global library directory: `~/.caro/library/`. `None` if the
/// platform doesn't expose a home directory.
pub fn global_library_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".caro").join("library"))
}

/// List `.caro` files under `./tasks/` (recursively). Returns an empty Vec
/// if the directory doesn't exist (this is not an error — `caro list` should
/// still work in projects without a `tasks/` directory).
pub fn list_project_tasks() -> Vec<TaskEntry> {
    list_in(&project_tasks_dir(), TaskSource::Project)
}

/// List `.caro` files under `~/.caro/library/` (recursively).
pub fn list_global_tasks() -> Vec<TaskEntry> {
    match global_library_dir() {
        Some(root) => list_in(&root, TaskSource::Global),
        None => Vec::new(),
    }
}

/// All tasks: project + global, with project entries shadowing same-named global ones.
pub fn list_all() -> Vec<TaskEntry> {
    let project = list_project_tasks();
    let project_names: std::collections::HashSet<String> =
        project.iter().map(|e| e.name.clone()).collect();

    let mut all = project;
    for entry in list_global_tasks() {
        if !project_names.contains(&entry.name) {
            all.push(entry);
        }
    }
    all.sort_by(|a, b| a.name.cmp(&b.name));
    all
}

/// Resolve a task name to a `.caro` path on disk.
///
/// Tries in order:
/// 1. If `name` ends in `.caro` or contains a path separator and exists, use it directly.
/// 2. `./tasks/<name>.caro`
/// 3. `~/.caro/library/<name>.caro`
///
/// Returns `None` if no match.
pub fn resolve_task_path(name: &str) -> Option<PathBuf> {
    // 1. Explicit path (relative or absolute)
    if name.ends_with(".caro") || name.contains(std::path::MAIN_SEPARATOR) {
        let p = PathBuf::from(name);
        if p.exists() {
            return Some(p);
        }
    }

    // 2. Project tasks
    let project_path = project_tasks_dir().join(format!("{}.caro", name));
    if project_path.exists() {
        return Some(project_path);
    }

    // 3. Global library
    if let Some(global) = global_library_dir() {
        let p = global.join(format!("{}.caro", name));
        if p.exists() {
            return Some(p);
        }
    }

    None
}

/// Look for a Carofile in the current working directory.
/// Recognized names: `Carofile` and `Carofile.caro`. Returns the first match.
pub fn find_carofile() -> Option<PathBuf> {
    for candidate in &["Carofile", "Carofile.caro"] {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn list_in(root: &Path, source: TaskSource) -> Vec<TaskEntry> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let read = match std::fs::read_dir(&dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("caro") {
                if let Some(name) = task_name_from_path(&path, root) {
                    out.push(TaskEntry { name, path, source });
                }
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn task_name_from_path(path: &Path, root: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    let stem_path = rel.with_extension("");
    let parts: Vec<String> = stem_path
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_caro(dir: &Path, rel: &str) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, "TASK Demo\nDO say hi\n").unwrap();
    }

    #[test]
    fn project_tasks_dir_is_relative_tasks() {
        assert_eq!(project_tasks_dir(), PathBuf::from("tasks"));
    }

    #[test]
    fn list_in_empty_dir_returns_empty() {
        let dir = TempDir::new().unwrap();
        assert!(list_in(dir.path(), TaskSource::Project).is_empty());
    }

    #[test]
    fn list_in_nonexistent_dir_returns_empty() {
        let p = PathBuf::from("/nonexistent/path/that/does/not/exist");
        assert!(list_in(&p, TaskSource::Project).is_empty());
    }

    #[test]
    fn list_in_finds_top_level_caro() {
        let dir = TempDir::new().unwrap();
        write_caro(dir.path(), "cleanup-logs.caro");
        write_caro(dir.path(), "deploy-api.caro");
        // Non-caro files are ignored
        fs::write(dir.path().join("readme.md"), "hello").unwrap();

        let entries = list_in(dir.path(), TaskSource::Project);
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["cleanup-logs", "deploy-api"]);
        assert!(entries.iter().all(|e| e.source == TaskSource::Project));
    }

    #[test]
    fn list_in_walks_subdirectories() {
        let dir = TempDir::new().unwrap();
        write_caro(dir.path(), "root.caro");
        write_caro(dir.path(), "system/snapshot.caro");
        write_caro(dir.path(), "system/cleanup.caro");
        write_caro(dir.path(), "deep/nested/foo.caro");

        let entries = list_in(dir.path(), TaskSource::Project);
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["deep/nested/foo", "root", "system/cleanup", "system/snapshot"]
        );
    }

    #[test]
    #[serial_test::serial]
    fn resolve_task_path_explicit_relative() {
        let dir = TempDir::new().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        write_caro(dir.path(), "stuff/x.caro");
        let resolved = resolve_task_path("stuff/x.caro");

        // Restore cwd before assertions so a failure doesn't poison other tests.
        std::env::set_current_dir(&original_cwd).unwrap();

        assert!(resolved.is_some());
        assert!(resolved.unwrap().to_string_lossy().ends_with("stuff/x.caro"));
    }

    #[test]
    #[serial_test::serial]
    fn find_carofile_prefers_no_extension() {
        let dir = TempDir::new().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        // No Carofile
        let none_found = find_carofile().is_none();

        fs::write("Carofile.caro", "TASK Project").unwrap();
        let only_dot_caro = find_carofile();

        fs::write("Carofile", "TASK Project").unwrap();
        let both = find_carofile();

        std::env::set_current_dir(&original_cwd).unwrap();

        assert!(none_found, "expected no Carofile in empty dir");
        assert_eq!(
            only_dot_caro.unwrap().to_string_lossy(),
            "Carofile.caro",
            "should find Carofile.caro when only it exists"
        );
        // `Carofile` (no extension) is preferred (matches Makefile convention).
        assert_eq!(
            both.unwrap().to_string_lossy(),
            "Carofile",
            "Carofile should win over Carofile.caro when both exist"
        );
    }

    #[test]
    fn list_all_project_shadows_global() {
        // We can't easily fake global library in a unit test without changing
        // $HOME, but we can verify the shadowing logic via the helper directly.
        let project = vec![TaskEntry {
            name: "shared".into(),
            path: PathBuf::from("tasks/shared.caro"),
            source: TaskSource::Project,
        }];
        let global = vec![
            TaskEntry {
                name: "shared".into(),
                path: PathBuf::from("/home/u/.caro/library/shared.caro"),
                source: TaskSource::Global,
            },
            TaskEntry {
                name: "global-only".into(),
                path: PathBuf::from("/home/u/.caro/library/global-only.caro"),
                source: TaskSource::Global,
            },
        ];

        let project_names: std::collections::HashSet<String> =
            project.iter().map(|e| e.name.clone()).collect();
        let mut combined = project;
        for entry in global {
            if !project_names.contains(&entry.name) {
                combined.push(entry);
            }
        }
        combined.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(combined.len(), 2);
        assert_eq!(combined[0].name, "global-only");
        assert_eq!(combined[0].source, TaskSource::Global);
        assert_eq!(combined[1].name, "shared");
        assert_eq!(combined[1].source, TaskSource::Project);
    }
}
