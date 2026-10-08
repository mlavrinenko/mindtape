//! The one list of modules that still print or build an error outside
//! [`crate::diag`], and the test that holds the source to it.
//!
//! A module bypasses the renderer in one of three ways: an
//! `#[allow(clippy::disallowed_macros)]` on its `mod` line, which lets it call
//! `eprintln!`, `bail!` or `anyhow!`; a `Diag::legacy(..)` call; or a direct
//! `std::io::stderr()`. Each such module is an entry below. The test refuses a
//! bypass that no entry covers, an entry that covers no bypass, and an entry
//! the branch point's list didn't have, so the list only shrinks: a fan-out
//! task converts a module and deletes its entry.

/// Paths under `crates/mindtape-cli/src/`: a module folder, or a file without
/// its `.rs`. Sorted.
const UNCONVERTED: &[&str] = &[
    "cli/check_argv",
    "cmd/add",
    "cmd/check",
    "cmd/eval",
    "cmd/flip",
    "cmd/go",
    "cmd/import",
    "cmd/init",
    "cmd/link",
    "cmd/ls",
    "cmd/mv",
    "cmd/preset",
    "cmd/recur",
    "cmd/review",
    "cmd/rm",
    "cmd/set",
    "cmd/worktrees",
    "pre_write",
    "task_edit",
];

#[path = "allowlist_tests.rs"]
#[allow(clippy::unwrap_used)]
mod tests;
