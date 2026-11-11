//! `path` and `dir`: the task fields that name where the task's file lives.
//!
//! A template's filled example and an `mt add` draft are tasks without a file
//! of their own, so a rule reading either field has no answer for them. The
//! template pass and the add-field whitelist both drop those rules by
//! [`path_dependent_fields`], so the two lists cannot drift apart.
//!
//! `dir` is the parent directory of `path`, read from the same file location.

/// Every filter field whose value comes from the task file's location: `path`
/// itself, and `dir`, the directory around it.
const PATH_DEPENDENT_FIELDS: [&str; 2] = ["path", "dir"];

/// [`PATH_DEPENDENT_FIELDS`], the one list every caller that judges a task
/// without a file reads.
#[must_use]
pub const fn path_dependent_fields() -> [&'static str; 2] {
    PATH_DEPENDENT_FIELDS
}
