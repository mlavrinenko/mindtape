//! `mt add --format`: report what a run did, in the caller's chosen shape.
//!
//! Mirrors `mt ls`'s `--format` vocabulary (`crate::output::OutputFormat`) so
//! a script learns one convention rather than two, but stays its own type:
//! `add` never lists many tasks, so `table`/`csv` have nothing to reuse, and
//! `text` — today's `wrote <path>` prose — has no `ls` equivalent, whose bare
//! default is the table. `--print` stays orthogonal: it previews the task
//! source before any write is attempted; this reports what a write (or a
//! `--dedup skip`/`add-suffix` decision) actually did — `wrote`, `skipped`,
//! or `suffixed` — so a dedup outcome is observable without parsing prose.

use std::path::{Path, PathBuf};

use serde_json::json;

/// `--format` on `mt add`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    /// `wrote <path>` / `skipped, ...` / `suffixed, ...` prose (the default).
    #[default]
    Text,
    /// The resulting path, nothing else.
    Path,
    /// The same path, NUL-terminated instead of newline-terminated.
    #[value(name = "path0")]
    Path0,
    /// `{"outcome": "wrote"|"skipped"|"suffixed", "path": "..."}`.
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

/// Render `outcome` in `format`. `Path0` is NUL-terminated and must reach the
/// terminal via `print!`, never `println!`; every other format wants the
/// caller's trailing newline (`println!`) — the same contract as
/// `crate::output::render_path`/`render_path0`.
#[must_use]
pub fn render(outcome: &Outcome, format: Format) -> String {
    match format {
        Format::Text => render_text(outcome),
        Format::Path => outcome.path().display().to_string(),
        Format::Path0 => format!("{}\0", outcome.path().display()),
        Format::Json => render_json(outcome),
    }
}

fn render_text(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Wrote(path) => format!("wrote {}", path.display()),
        Outcome::Skipped(path) => format!("skipped, already exists at {}", path.display()),
        Outcome::Suffixed(path) => format!("suffixed, wrote {}", path.display()),
    }
}

fn render_json(outcome: &Outcome) -> String {
    let value = json!({
        "outcome": outcome.label(),
        "path": outcome.path().display().to_string(),
    });
    serde_json::to_string_pretty(&value).unwrap_or_default()
}

/// Print `outcome` in `format` to stdout, honouring `Path0`'s no-trailing-
/// newline contract.
pub fn emit(outcome: &Outcome, format: Format) {
    let rendered = render(outcome, format);
    if format == Format::Path0 {
        print!("{rendered}");
    } else {
        println!("{rendered}");
    }
}

#[cfg(test)]
#[path = "add_output_tests.rs"]
mod tests;
