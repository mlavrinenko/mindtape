//! What `mt fix` prints for each import it found, and the closing count.

use std::path::Path;

use crate::render::widest;

/// One report row: `kept <file> <version>` or
/// `rewrite <file> <old> -> <new>`.
pub(super) struct Line {
    verb: &'static str,
    file: String,
    detail: String,
}

impl Line {
    /// A supported older pin that stays as it is.
    pub(super) fn kept(file: &Path, version: &str) -> Self {
        Self {
            verb: "kept",
            file: mindtape::display_path(file),
            detail: version.to_owned(),
        }
    }

    /// An unsupported older pin the fixer would change.
    pub(super) fn rewrite(file: &Path, version: &str) -> Self {
        Self {
            verb: "rewrite",
            file: mindtape::display_path(file),
            detail: format!("{version} -> {}", mindtape::PRELUDE_VERSION),
        }
    }

    /// Whether this row is an edit, not a kept pin.
    pub(super) fn is_rewrite(&self) -> bool {
        self.verb == "rewrite"
    }
}

/// The counts the closing line states.
pub(super) struct Tally {
    /// Imports the fixer edits.
    pub(super) rewrites: usize,
    /// Supported older pins the fixer leaves alone.
    pub(super) kept: usize,
}

/// Print `lines` as one aligned row each, and return their counts.
pub(super) fn print(lines: &[Line]) -> Tally {
    let width = widest(lines.iter().map(|line| line.file.as_str()));
    let mut tally = Tally {
        rewrites: 0,
        kept: 0,
    };
    for line in lines {
        if line.is_rewrite() {
            tally.rewrites += 1;
        } else {
            tally.kept += 1;
        }
        println!("{:<8}{:<width$}  {}", line.verb, line.file, line.detail);
    }
    tally
}

/// The closing line: `would rewrite 1 import; 1 kept`, or
/// `nothing to rewrite`.
pub(super) fn summary(verb: &str, tally: &Tally) -> String {
    let head = if tally.rewrites == 0 {
        "nothing to rewrite".to_owned()
    } else {
        format!("{verb} {}", mindtape::count(tally.rewrites, "import"))
    };
    if tally.kept == 0 {
        head
    } else {
        format!("{head}; {} kept", tally.kept)
    }
}
