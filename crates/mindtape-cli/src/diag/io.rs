//! A read or write that failed, worded the one way: the path single-quoted and
//! the operating system's reason after it.
//!
//! ```text
//! error: can't read: 'nope.typ': no such file or directory
//! error: can't write: 'tasks/a.typ': permission denied
//! ```
//!
//! A command that touches a file uses [`Diag::cannot_read`] or
//! [`Diag::cannot_write`] and adds a hint when it has a next step. Any other
//! message that fails on I/O ends in [`Diag::io`], so the reason reads the same
//! everywhere: the OS's words, lowercase, with Rust's `(os error 2)` left off.

use std::io;
use std::path::Path;

use super::Diag;

impl Diag {
    /// The file at `path` could not be read: `can't read: '<path>': <reason>`.
    pub(crate) fn cannot_read(path: &Path, err: &io::Error) -> Self {
        Self::failed("can't read")
            .value(path.display().to_string())
            .io(err)
    }

    /// The file at `path` could not be written: `can't write: '<path>':
    /// <reason>`. Nothing was written.
    pub(crate) fn cannot_write(path: &Path, err: &io::Error) -> Self {
        Self::failed("can't write")
            .value(path.display().to_string())
            .io(err)
    }

    /// Why it failed, when the OS said so: `err` as a reason. Use it for an
    /// I/O failure with a message of its own, such as `can't resolve path`.
    pub(crate) fn io(self, err: &io::Error) -> Self {
        self.reason(reason(err))
    }
}

/// `err` as the reason of a message: its text with a trailing `(os error N)`
/// dropped and the first letter lowercase.
fn reason(err: &io::Error) -> String {
    let text = err.to_string();
    let text = match text.rfind(" (os error ") {
        Some(at) if text.ends_with(')') => text.get(..at).unwrap_or(&text),
        _ => &text,
    };
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

impl From<io::Error> for Diag {
    /// An I/O failure no command words itself: a terminal, a pipe or a buffer.
    fn from(err: io::Error) -> Self {
        Self::failed("i/o failed").io(&err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diag::render::render;

    #[test]
    fn a_bare_io_error_says_i_o_failed_with_the_reason() {
        let diag = Diag::from(io::Error::other("Broken pipe"));
        assert_eq!(render(&diag, false), "error: i/o failed: broken pipe\n");
    }

    #[test]
    fn an_error_with_no_text_gives_an_empty_reason() {
        assert_eq!(reason(&io::Error::other("")), "");
    }

    #[test]
    fn the_os_error_suffix_is_dropped_and_the_first_letter_lowercased() {
        let err = io::Error::from_raw_os_error(2);
        assert_eq!(reason(&err), "no such file or directory");
    }
}
