//! Tests for status-value rendering and prelude-shortcut drift.
#![allow(clippy::unwrap_used)]

use super::{STATUS_SHORTCUTS, status_value};

use crate::test_support::PRELUDE;

#[test]
fn shortcut_name_uses_bare_constructor() {
    assert_eq!(
        status_value("done", (2026, 6, 28), None),
        "done(2026, 6, 28)"
    );
}

#[test]
fn custom_name_uses_verbose_form() {
    assert_eq!(
        status_value("icebox", (2026, 6, 28), None),
        "status(\"icebox\", 2026, 6, 28)"
    );
}

#[test]
fn shortcut_note_rides_as_content_block() {
    assert_eq!(
        status_value("wip", (2026, 6, 28), Some("on it")),
        "wip(2026, 6, 28)[on it]"
    );
}

#[test]
fn verbose_note_rides_as_named_arg() {
    assert_eq!(
        status_value("icebox", (2026, 6, 28), Some("later")),
        "status(\"icebox\", 2026, 6, 28, note: [later])"
    );
}

#[test]
fn blank_note_is_dropped() {
    assert_eq!(
        status_value("done", (2026, 6, 28), Some("   ")),
        "done(2026, 6, 28)"
    );
}

#[test]
fn note_markup_is_escaped() {
    assert_eq!(
        status_value("done", (2026, 6, 28), Some("see [#1] *now*")),
        "done(2026, 6, 28)[see \\[\\#1\\] \\*now\\*]"
    );
}

#[test]
fn shortcuts_match_prelude() {
    // Every `_mk("…")` binding in the prelude must appear in STATUS_SHORTCUTS and
    // vice versa, so the shortcut rendering never names a function the prelude
    // lacks (nor misses one it ships).
    let from_prelude: std::collections::BTreeSet<&str> = PRELUDE
        .lines()
        .filter_map(|line| line.trim().strip_prefix("#let "))
        .filter_map(|rest| rest.split_once("= _mk(\""))
        .filter_map(|(_, tail)| tail.split_once("\")"))
        .map(|(name, _)| name)
        .collect();
    let from_const: std::collections::BTreeSet<&str> = STATUS_SHORTCUTS.iter().copied().collect();
    assert_eq!(from_const, from_prelude);
}
