//! Unit tests for [`Legend::initial`] and the transition graph.
#![allow(clippy::unwrap_used)]

use super::{Legend, LegendEntry};
use crate::link::NoteRule;

fn entry(value: &str, terminal: bool) -> LegendEntry {
    LegendEntry {
        value: value.to_owned(),
        terminal,
        ..Default::default()
    }
}

/// An entry with declared `to` edges (non-terminal).
fn edge(value: &str, to: &[&str]) -> LegendEntry {
    LegendEntry {
        value: value.to_owned(),
        to: to.iter().map(|s| (*s).to_owned()).collect(),
        ..Default::default()
    }
}

/// The dogfood-shaped lifecycle: proposed → accepted/wip/cancelled, wip →
/// done/failed/accepted, terminal done/failed reopen to wip, cancelled a sink.
fn lifecycle() -> Legend {
    Legend {
        entries: vec![
            edge("proposed", &["accepted", "wip", "cancelled"]),
            edge("accepted", &["wip", "cancelled"]),
            edge("wip", &["done", "failed", "accepted"]),
            LegendEntry {
                terminal: true,
                to: vec!["wip".to_owned()],
                ..entry("done", true)
            },
            entry("cancelled", true),
            LegendEntry {
                terminal: true,
                to: vec!["wip".to_owned()],
                ..entry("failed", true)
            },
        ],
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

#[test]
fn transitions_declared_only_when_an_entry_has_to() {
    assert!(!Legend::default().transitions_declared());
    let flat = Legend {
        entries: vec![entry("a", false), entry("done", true)],
    };
    assert!(!flat.transitions_declared());
    assert!(lifecycle().transitions_declared());
}

#[test]
fn can_transition_follows_declared_edges() {
    let legend = lifecycle();
    assert!(legend.can_transition("proposed", "wip"));
    assert!(legend.can_transition("done", "wip")); // reopen edge
    assert!(!legend.can_transition("proposed", "done"));
    assert!(!legend.can_transition("cancelled", "wip")); // sink
    assert!(!legend.can_transition("unknown", "wip"));
}

#[test]
fn allowed_next_lists_edges_in_order() {
    let legend = lifecycle();
    assert_eq!(legend.allowed_next("wip"), ["done", "failed", "accepted"]);
    assert!(legend.allowed_next("cancelled").is_empty());
    assert!(legend.allowed_next("unknown").is_empty());
}

#[test]
fn reachable_spans_the_whole_lifecycle() {
    let reachable = lifecycle().reachable();
    for value in ["proposed", "accepted", "wip", "done", "failed", "cancelled"] {
        assert!(reachable.contains(value), "{value} should be reachable");
    }
}

#[test]
fn reachable_excludes_an_orphan_state() {
    // `blocked` has no incoming edge, so nothing reaches it from the initial.
    let legend = Legend {
        entries: vec![
            edge("proposed", &["done"]),
            entry("blocked", false),
            entry("done", true),
        ],
    };
    let reachable = legend.reachable();
    assert!(reachable.contains("proposed"));
    assert!(reachable.contains("done"));
    assert!(!reachable.contains("blocked"));
}

#[test]
fn reachable_is_empty_without_an_initial_state() {
    let legend = Legend {
        entries: vec![LegendEntry {
            terminal: true,
            to: vec!["done".to_owned()],
            ..entry("done", true)
        }],
    };
    assert!(legend.reachable().is_empty());
}

#[test]
fn transition_problems_flags_an_unknown_endpoint() {
    let legend = Legend {
        entries: vec![
            edge("proposed", &["accepted", "typo"]),
            entry("accepted", false),
        ],
    };
    let problems = legend.transition_problems();
    assert_eq!(problems.len(), 1);
    assert!(problems.first().is_some_and(|p| p.contains("`typo`")));
}

#[test]
fn transition_problems_empty_for_a_sound_graph() {
    assert!(lifecycle().transition_problems().is_empty());
    assert!(Legend::default().transition_problems().is_empty());
}

#[test]
fn note_defaults_to_optional() {
    assert_eq!(LegendEntry::default().note, NoteRule::Optional);
    assert!(!LegendEntry::default().note_required());
}

#[test]
fn note_required_true_only_for_the_required_variant() {
    let mut e = entry("blocked", false);
    assert!(!e.note_required());
    e.note = NoteRule::Forbidden;
    assert!(!e.note_required());
    e.note = NoteRule::Required;
    assert!(e.note_required());
}
