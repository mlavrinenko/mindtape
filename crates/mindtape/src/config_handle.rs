//! `[handle]` config: how many check characters a list handle carries.
//!
//! A handle is `<ordinal><check>`: the ordinal is the task's position in the
//! default view, recomputed on every command, and the check is derived from the
//! task's stable id. The ordinal is what goes stale — a flip re-sorts the view
//! under a caller still holding a handle — so the check is the only thing
//! standing between a stale handle and an operation on the wrong task.
//!
//! One base-26 letter lets a wrong handle through about once in 26. That is a
//! per-project tradeoff between how short a handle is to type and how much a
//! mistyped or stale one is caught, which is why it is a config key rather than
//! a constant: a two-person backlog driven by hand wants the short form, and a
//! project whose flips are scripted wants the guard.
//!
//! Omitted, this inherits the CLI's own default (one character), so a project
//! that says nothing keeps the handles it has always had.

use serde::Deserialize;

/// The largest check length FNV-1a's 32 bits can fill with real entropy.
///
/// `check_chars` peels one base-26 digit per character, so the useful ceiling
/// is `floor(log26(2^32))` = 6; a seventh character would be `'a'` for all but
/// a rounding-error slice of ids and would read as a guard it is not.
pub const MAX_CHECK_LEN: usize = 6;

/// `[handle]` — the shape of a `mt ls` handle.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct HandleConfig {
    /// How many check characters follow a handle's ordinal; omitted → 1.
    /// A handle of any other length is refused rather than prefix-matched, so
    /// raising this invalidates the short handles rather than accepting them.
    pub check_len: Option<usize>,
}

/// Reject a `check_len` outside `1..=MAX_CHECK_LEN`.
///
/// Zero would drop the check entirely, leaving a bare ordinal that any stale
/// listing satisfies; past [`MAX_CHECK_LEN`] the extra characters are constant.
pub(crate) fn validate_handle(config: &crate::Config) -> Vec<String> {
    match config.handle.check_len {
        Some(0) => vec![
            "handle.check-len: must be at least 1 — a handle with no check is a bare ordinal, \
             which every stale listing satisfies"
                .to_owned(),
        ],
        Some(len) if len > MAX_CHECK_LEN => vec![format!(
            "handle.check-len: {len} exceeds the {MAX_CHECK_LEN} characters a 32-bit hash can \
             fill; the extra characters would be constant"
        )],
        _ => vec![],
    }
}

#[cfg(test)]
#[path = "config_handle_tests.rs"]
mod tests;
