//! Deterministic benchmark fixtures: N task files in a throwaway Git repo.
//!
//! `mt` discovers tasks through Git, so a fixture that is not a work tree
//! measures the fallback walk instead of the path a user runs. Each fixture is
//! its own repository, built once under `target/bench/` and reused until the
//! task count changes. Every varying value derives from the file index. The
//! same N reproduces byte-identical files.
//!
//! The project config is `fixture/config.toml`, read at compile time: one
//! benchmark project, one source of truth.
#![allow(clippy::disallowed_methods, clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The benchmark project's config.
const CONFIG: &str = include_str!("fixture/config.toml");

/// The workspace `target/` directory, honoring `CARGO_TARGET_DIR`.
fn target_dir() -> PathBuf {
    match std::env::var_os("CARGO_TARGET_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target"),
    }
}

/// The fixture holding `n` tasks, built on first use and reused after.
///
/// # Panics
/// Panics when the fixture fails to build, either on a write or on a Git
/// call. A benchmark fixture that will not build is a harness bug, not a case
/// to recover from.
pub fn fixture(n: usize) -> PathBuf {
    let dir = target_dir().join("bench").join(format!("tasks-{n}"));
    if !is_built(&dir, n) {
        build(&dir, n);
    }
    dir
}

/// Whether `dir` already holds the `n`-task fixture.
fn is_built(dir: &Path, n: usize) -> bool {
    dir.join(".git").is_dir() && dir.join("tasks").join(stem(n)).is_file()
}

/// Write the fixture from scratch: config, `n` task files, one index.
fn build(dir: &Path, n: usize) {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir.join("tasks")).unwrap();
    fs::create_dir_all(dir.join(".mindtape")).unwrap();
    fs::write(dir.join(".mindtape/config.toml"), CONFIG).unwrap();
    for index in 1..=n {
        let path = dir.join("tasks").join(stem(index));
        fs::write(path, task_source(index)).unwrap();
    }
    // `mt` reads the index (`git ls-files --cached`), so staging is enough: no
    // commit, no identity, no signing. The developer's own Git configuration
    // stays out of reach, so a global `core.excludesFile` cannot hide a task.
    for args in [["init", "--quiet"], ["add", "--all"]] {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", dir.join("no-global-git-config"))
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    }
}

/// `task-00001.typ` for `index`.
fn stem(index: usize) -> String {
    format!("task-{index:05}.typ")
}

/// One task file. Statuses stay non-terminal so strict ICE priority
/// (`required = "!terminal"`) always holds, and every third task links to the
/// one before it, building a linear acyclic chain.
fn task_source(index: usize) -> String {
    let status = match index % 3 {
        0 => "proposed",
        1 => "accepted",
        _ => "wip",
    };
    let links = if index > 1 && index.is_multiple_of(3) {
        format!("  links: depends-on(\"{}\"),\n", stem(index - 1))
    } else {
        String::new()
    };
    format!(
        "#import \"@local/mindtape:{version}\": *\n\
         \n\
         #show: task.with(\n\
         \x20 title: \"Bench task {index}\",\n\
         \x20 priority: ice(\n\
         \x20   impact: {impact},\n\
         \x20   confidence: {confidence:.2},\n\
         \x20   ease: {ease},\n\
         \x20 ),\n\
         \x20 tags: (\"bench\",),\n\
         {links}\
         \x20 status: {status}(2026, 7, 16),\n\
         )\n\
         \n\
         == Summary\n\
         \n\
         Generated benchmark task {index}. Synthetic fixture task for \
         load-testing `mt ls` and `mt check` at scale. Task index {index} of \
         the deterministic bench batch.\n",
        version = mindtape::PRELUDE_VERSION,
        impact = index % 9 + 1,
        confidence = 0.5 + (index % 9) as f64 * 0.05,
        ease = (index * 3) % 9 + 1,
    )
}

/// Empty the harvest cache. The next load evaluates every file.
pub fn clear_cache(root: &Path) {
    let _ = fs::remove_dir_all(root.join(".mindtape/cache"));
}
