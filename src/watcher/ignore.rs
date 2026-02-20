//! Gitignore and `.mindtapeignore` pattern matching for the watcher.

use std::path::{Path, PathBuf};

use ignore::gitignore::{Gitignore, GitignoreBuilder};

/// Build a `Gitignore` matcher for a file path, collecting ignore rules from
/// the watch root down to the file's parent directory. Called on each event
/// so that changes to ignore files are picked up without restarting.
///
/// Walks from `root` down to the file's parent, loading `.gitignore` and
/// `.mindtapeignore` at each level (deeper files override shallower ones).
/// Global gitignore has the lowest precedence.
pub fn build_ignore(root: &Path, file: &Path) -> Gitignore {
    let mut builder = GitignoreBuilder::new(root);

    // Global gitignore (lowest precedence).
    if let Some(global) = global_gitignore_path()
        && global.exists()
    {
        builder.add(&global);
    }

    // Collect directories from root down to the file's parent.
    let target = file.parent().unwrap_or(root);
    let mut dirs_to_check: Vec<&Path> = Vec::new();
    let mut current = target;
    loop {
        dirs_to_check.push(current);
        if current == root {
            break;
        }
        match current.parent() {
            Some(parent) if parent != current => current = parent,
            _ => break,
        }
    }
    // Reverse so we go from root (lower precedence) to deepest dir (higher).
    dirs_to_check.reverse();

    for dir in dirs_to_check {
        let gitignore = dir.join(".gitignore");
        if gitignore.exists() {
            builder.add(&gitignore);
        }
        let mindtapeignore = dir.join(".mindtapeignore");
        if mindtapeignore.exists() {
            builder.add(&mindtapeignore);
        }
    }

    builder.build().unwrap_or_else(|_| Gitignore::empty())
}

/// Find the global gitignore file path.
///
/// Checks `git config --global core.excludesFile` first, then falls back
/// to the XDG-compliant default (`$XDG_CONFIG_HOME/git/ignore` or
/// `~/.config/git/ignore`).
fn global_gitignore_path() -> Option<PathBuf> {
    // Try git config first.
    if let Ok(output) = std::process::Command::new("git")
        .args(["config", "--global", "core.excludesFile"])
        .output()
        && output.status.success()
    {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(crate::config::expand_tilde(&path));
        }
    }

    // XDG fallback.
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("git/ignore"));
    }

    directories::BaseDirs::new().map(|d| d.config_dir().join("git/ignore"))
}
