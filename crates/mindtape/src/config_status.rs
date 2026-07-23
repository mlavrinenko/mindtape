//! `[[tasks.status]]` — the status legend: an ordered, displayable vocabulary.
//!
//! Split out of `config.rs` to keep that file under the size gate; re-exported
//! at `crate::config::{Legend, LegendEntry}`.

use std::collections::{HashSet, VecDeque};

use serde::Deserialize;

use crate::link::NoteRule;

/// The status legend: an ordered, displayable vocabulary.
///
/// Each entry binds a value to optional display attributes (glyph, colour,
/// label) plus the `terminal` flag. The legend defines the legal status
/// values, their lifecycle order, and how each renders. An empty legend is
/// unconstrained, though omitting `[[tasks.status]]` entirely still inherits
/// `MindTape`'s shipped six-value legend.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(transparent)]
pub struct Legend {
    /// Ordered entries, in lifecycle / display order.
    pub entries: Vec<LegendEntry>,
}

/// One `[[tasks.status]]` entry: a value plus its display attributes.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct LegendEntry {
    /// The value this entry describes (e.g. a status name).
    pub value: String,
    /// Optional decorative glyph (emoji or ASCII).
    pub glyph: Option<String>,
    /// Optional colour: a named ANSI colour or `#RRGGBB`.
    pub color: Option<String>,
    /// Optional display label; falls back to `value`.
    pub label: Option<String>,
    /// Whether this value ends a task; required fields relax once terminal.
    pub terminal: bool,
    /// Allowed next statuses when leaving this one — the outgoing edges of the
    /// transition graph. Empty on every entry keeps the legend any-to-any
    /// (unconstrained). Once any entry declares `to`, transitions are enforced
    /// and an empty `to` marks a sink: nothing out but a same-status re-stamp.
    /// A reopen edge is just a terminal (or any) state naming its target here.
    pub to: Vec<String>,
    /// Note requirement for a task landing in this status: whether the change
    /// must (`required`), may (`optional`, the default), or must not
    /// (`forbidden`) carry a note. Set per status rather than legend-wide, so
    /// a project can demand a note for statuses with no automated check (e.g.
    /// `blocked`, `failed`) while leaving others unconstrained.
    pub note: NoteRule,
}

impl LegendEntry {
    /// Whether this entry's [`NoteRule`] is `Required` — the only variant
    /// enforced for a status (unlike a link kind, a status never rejects a
    /// present note, so `Forbidden` is not checked here).
    #[must_use]
    pub fn note_required(&self) -> bool {
        self.note == NoteRule::Required
    }
}

impl Legend {
    /// Whether the legend constrains nothing (no entries).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether `value` is a legal entry.
    #[must_use]
    pub fn contains(&self, value: &str) -> bool {
        self.entries.iter().any(|entry| entry.value == value)
    }

    /// The entry describing `value`, if any.
    #[must_use]
    pub fn entry(&self, value: &str) -> Option<&LegendEntry> {
        self.entries.iter().find(|entry| entry.value == value)
    }

    /// The values flagged `terminal`.
    #[must_use]
    pub fn terminal(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter(|entry| entry.terminal)
            .map(|entry| entry.value.clone())
            .collect()
    }

    /// The natural starting status for a freshly filed task: the first entry
    /// in lifecycle order with `terminal == false`. `None` when every entry is
    /// terminal (unusual) or the legend is empty (a project that cleared it
    /// with `status = []`) — callers must not invent a value in that case.
    #[must_use]
    pub fn initial(&self) -> Option<&LegendEntry> {
        self.entries.iter().find(|entry| !entry.terminal)
    }

    /// Whether the transition graph is active: at least one entry declares a
    /// `to` list. False means unconstrained (any-to-any), exactly as before the
    /// feature existed — so an absent graph never gates a flip.
    #[must_use]
    pub fn transitions_declared(&self) -> bool {
        self.entries.iter().any(|entry| !entry.to.is_empty())
    }

    /// Whether `from → to` is a legal edge. Only meaningful when
    /// [`transitions_declared`](Self::transitions_declared); a same-status
    /// re-stamp (`from == to`) is always legal and handled by the caller.
    #[must_use]
    pub fn can_transition(&self, from: &str, to: &str) -> bool {
        self.entry(from)
            .is_some_and(|entry| entry.to.iter().any(|next| next == to))
    }

    /// The allowed next statuses when leaving `from`, in declared order. Empty
    /// slice for an unknown status or a declared sink.
    #[must_use]
    pub fn allowed_next(&self, from: &str) -> &[String] {
        self.entry(from).map_or(&[], |entry| &entry.to)
    }

    /// The statuses reachable from the initial state by walking `to` edges
    /// (the initial state included). Used to flag structurally dead states: a
    /// task whose current status is outside this set can only have arrived by a
    /// hand-edit or under a since-changed graph. Empty when there is no initial
    /// state (every entry terminal, or the legend cleared).
    #[must_use]
    pub fn reachable(&self) -> HashSet<String> {
        let mut seen = HashSet::new();
        let Some(start) = self.initial() else {
            return seen;
        };
        let mut queue = VecDeque::from([start.value.clone()]);
        while let Some(value) = queue.pop_front() {
            if !seen.insert(value.clone()) {
                continue;
            }
            for next in self.allowed_next(&value) {
                if !seen.contains(next) {
                    queue.push_back(next.clone());
                }
            }
        }
        seen
    }

    /// Well-formedness problems in the declared graph: any `to` endpoint that is
    /// not itself a legend value. One message per bad edge; empty when the graph
    /// is sound (or absent).
    #[must_use]
    pub fn transition_problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        for entry in &self.entries {
            for next in &entry.to {
                if !self.contains(next) {
                    problems.push(format!(
                        "status.transitions: `{}` -> `{next}`: `{next}` is not a configured status",
                        entry.value
                    ));
                }
            }
        }
        problems
    }
}

#[cfg(test)]
#[path = "config_status_tests.rs"]
mod tests;
