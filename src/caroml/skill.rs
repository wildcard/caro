//! Caro skill installer — copies the bundled `caro-scaffold` skill into the
//! local skill-aware coder agent's directory (default `~/.claude/skills/`).
//!
//! The bundled skill source lives under `.claude/skills/caro-scaffold/` in
//! this repo and is read at install-time. v0.1 ships the skill files as
//! committed text under `.claude/skills/`; v0.2 may embed them via
//! `include_str!` for static binaries.

use std::path::{Path, PathBuf};
use thiserror::Error;

const SKILL_NAME: &str = "caro-scaffold";

/// Source files that make up the skill (relative to the project root's `.claude/skills/caro-scaffold/`).
const SKILL_FILES: &[&str] = &["SKILL.md", "README.md"];

#[derive(Debug, Error)]
pub enum SkillError {
    #[error("home directory not available")]
    NoHomeDir,
    #[error("skill source not found at {0}")]
    SourceMissing(PathBuf),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// The default install destination: `<home>/.claude/skills/caro-scaffold/`.
pub fn default_install_dir() -> Result<PathBuf, SkillError> {
    let home = dirs::home_dir().ok_or(SkillError::NoHomeDir)?;
    Ok(home.join(".claude").join("skills").join(SKILL_NAME))
}

/// Where the skill source ships in this repo.
pub fn bundled_source_dir() -> PathBuf {
    PathBuf::from(".claude").join("skills").join(SKILL_NAME)
}

/// Install the skill: copy each file in [`SKILL_FILES`] from `source_dir`
/// to `dest_dir`. Returns the final `dest_dir`.
pub fn install(source_dir: &Path, dest_dir: &Path) -> Result<PathBuf, SkillError> {
    if !source_dir.exists() {
        return Err(SkillError::SourceMissing(source_dir.to_path_buf()));
    }
    std::fs::create_dir_all(dest_dir)?;
    for filename in SKILL_FILES {
        let src = source_dir.join(filename);
        if !src.exists() {
            // Skip optional files; SKILL.md is required, README.md is best-effort.
            if *filename == "SKILL.md" {
                return Err(SkillError::SourceMissing(src));
            }
            continue;
        }
        let dst = dest_dir.join(filename);
        std::fs::copy(&src, &dst)?;
    }
    Ok(dest_dir.to_path_buf())
}

/// Uninstall: remove `dest_dir` recursively. No-op if missing.
pub fn uninstall(dest_dir: &Path) -> Result<bool, SkillError> {
    if !dest_dir.exists() {
        return Ok(false);
    }
    std::fs::remove_dir_all(dest_dir)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill_source(root: &Path) -> PathBuf {
        let dir = root.join("source/.claude/skills/caro-scaffold");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("SKILL.md"),
            "---\nname: caro-scaffold\n---\nbody",
        )
        .unwrap();
        std::fs::write(dir.join("README.md"), "# caro-scaffold\n").unwrap();
        dir
    }

    #[test]
    fn install_copies_skill_md_and_readme() {
        let temp = tempfile::tempdir().unwrap();
        let source = make_skill_source(temp.path());
        let dest = temp.path().join("dest/skills/caro-scaffold");
        install(&source, &dest).unwrap();
        assert!(dest.join("SKILL.md").exists());
        assert!(dest.join("README.md").exists());
        let body = std::fs::read_to_string(dest.join("SKILL.md")).unwrap();
        assert!(body.contains("name: caro-scaffold"));
    }

    #[test]
    fn install_errors_when_source_missing() {
        let temp = tempfile::tempdir().unwrap();
        let dest = temp.path().join("dest");
        match install(&temp.path().join("nope"), &dest) {
            Err(SkillError::SourceMissing(_)) => {}
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    fn uninstall_removes_dir() {
        let temp = tempfile::tempdir().unwrap();
        let source = make_skill_source(temp.path());
        let dest = temp.path().join("dest");
        install(&source, &dest).unwrap();
        assert!(dest.exists());
        assert!(uninstall(&dest).unwrap());
        assert!(!dest.exists());
    }

    #[test]
    fn uninstall_missing_returns_false() {
        let temp = tempfile::tempdir().unwrap();
        let result = uninstall(&temp.path().join("never-existed")).unwrap();
        assert!(!result);
    }

    #[test]
    fn install_skips_missing_optional_readme() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("src");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("SKILL.md"), "---\nname: x\n---").unwrap();
        // No README — should still succeed.
        let dest = temp.path().join("dest");
        install(&source, &dest).unwrap();
        assert!(dest.join("SKILL.md").exists());
        assert!(!dest.join("README.md").exists());
    }
}
