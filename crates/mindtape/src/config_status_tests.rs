//! Unit tests for [`Legend::initial`].
#![allow(clippy::unwrap_used)]

use super::{Legend, LegendEntry};

fn entry(value: &str, terminal: bool) -> LegendEntry {
    LegendEntry {
        value: value.to_owned(),
        terminal,
        ..Default::default()
    }
}

#[test]
fn initial_is_first_non_terminal_entry() {
    let legend = Legend {
        entries: vec![entry("a", false), entry("b", false), entry("done", true)],
    };
    assert_eq!(legend.initial().map(|e| e.value.as_str()), Some("a"));
}

#[test]
fn initial_skips_a_terminal_first_entry() {
    let legend = Legend {
        entries: vec![entry("done", true), entry("open", false)],
    };
    assert_eq!(legend.initial().map(|e| e.value.as_str()), Some("open"));
}

#[test]
fn initial_is_none_when_every_entry_is_terminal() {
    let legend = Legend {
        entries: vec![entry("done", true), entry("cancelled", true)],
    };
    assert_eq!(legend.initial(), None);
}

#[test]
fn initial_is_none_for_an_empty_legend() {
    assert_eq!(Legend::default().initial(), None);
}
