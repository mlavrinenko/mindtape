//! `mt add --format`: report what a run did, in the caller's chosen shape.
//!
//! Mirrors `mt ls`'s `--format` vocabulary (`crate::render::output::OutputFormat`) so
//! a script learns one convention rather than two, but stays its own type:
//! `add` never lists many tasks, so `table`/`csv` have nothing to reuse, and
//! `text` — the `wrote <path> (<hash>)` prose — has no `ls` equivalent, whose bare
//! default is the table. `--print` stays orthogonal: it previews the task
//! source before any write is attempted; this reports what a write (or a
//! `--dedup skip`/`add-suffix` decision) actually did — `wrote`, `skipped`,
//! or `suffixed` — so a dedup outcome is observable without parsing prose.

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::render::file_path;
use crate::util;

/// `--format` on `mt add`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    /// A sentence such as `wrote <path> (<slug hash>)`.
    #[default]
    Text,
    /// Just the task's path.
    Path,
    /// The path, ending in a NUL byte instead of a newline.
    #[value(name = "path0")]
    Path0,
    /// A JSON object with `outcome` (`wrote`, `skipped` or `suffixed`), `path` and `slug`.
    Json,
}

/// What `mt add` did about the destination stem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Wrote a fresh file at this path.
    Wrote(PathBuf),
    /// `--dedup skip` found this path already taken; nothing was written.
    Skipped(PathBuf),
    /// `--dedup add-suffix` minted this disambiguated path and wrote there.
    Suffixed(PathBuf),
}

impl Outcome {
    /// The path this outcome is about — written, existing, or suffixed.
    #[must_use]
    pub fn path(&self) -> &Path {
        match self {
            Self::Wrote(path) | Self::Skipped(path) | Self::Suffixed(path) => path,
        }
    }

    /// The `outcome` value machine readers key on: `wrote`, `skipped`, or
    /// `suffixed`.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Wrote(_) => "wrote",
            Self::Skipped(_) => "skipped",
            Self::Suffixed(_) => "suffixed",
        }
    }
}

/// How the task is named to a reader: its slug hash, which every command
/// accepts as a reference, and its path as the prose report shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handle {
    /// The task's stem hash, as `mt ls` shows it in the `slug#` column.
    pub hash: String,
    /// The path the text report prints: relative to the working directory.
    pub shown: String,
}

/// The [`Handle`] of the task `outcome` is about, hashed among the project's
/// loaded stems with [`util::stem_hashes_with`], the function `mt ls` uses.
#[must_use]
pub fn handle(project: &util::Project, outcome: &Outcome) -> Handle {
    let path = outcome.path();
    let stem = path.file_stem().and_then(|stem| stem.to_str());
    let hashes = util::stem_hashes_with(&project.outcome.tasks, &project.config, stem);
    let shown = match path.strip_prefix(&project.root) {
        Ok(rel) => file_path::cell_text(&project.root, rel),
        Err(_) => path.display().to_string(),
    };
    Handle {
        hash: stem
            .and_then(|stem| hashes.hash_of(stem))
            .unwrap_or_default()
            .to_owned(),
        shown,
    }
}

/// Render `outcome` in `format`. `Path0` is NUL-terminated and must reach the
/// terminal via `print!`, never `println!`; every other format wants the
/// caller's trailing newline (`println!`) — the same contract as
/// `crate::render::output::render_path`/`render_path0`.
#[must_use]
pub fn render(outcome: &Outcome, handle: &Handle, format: Format) -> String {
    match format {
        Format::Text => render_text(outcome, handle),
        Format::Path => outcome.path().display().to_string(),
        Format::Path0 => format!("{}\0", outcome.path().display()),
        Format::Json => render_json(outcome, handle),
    }
}

fn render_text(outcome: &Outcome, handle: &Handle) -> String {
    let Handle { hash, shown } = handle;
    match outcome {
        Outcome::Wrote(_) => format!("wrote {shown} ({hash})"),
        Outcome::Skipped(_) => format!("skipped, already exists at {shown} ({hash})"),
        Outcome::Suffixed(_) => format!("suffixed, wrote {shown} ({hash})"),
    }
}

fn render_json(outcome: &Outcome, handle: &Handle) -> String {
    let value = json!({
        "outcome": outcome.label(),
        "path": outcome.path().display().to_string(),
        "slug": handle.hash,
    });
    serde_json::to_string_pretty(&value).unwrap_or_default()
}

/// Print `outcome` in `format` to stdout, honouring `Path0`'s no-trailing-
/// newline contract.
pub fn emit(outcome: &Outcome, handle: &Handle, format: Format) {
    let rendered = render(outcome, handle, format);
    if format == Format::Path0 {
        print!("{rendered}");
    } else {
        println!("{rendered}");
    }
}

#[cfg(test)]
#[path = "output_tests.rs"]
mod tests;
