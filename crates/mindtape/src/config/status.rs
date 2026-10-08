//! `[[tasks.status]]` — the status legend: an ordered, displayable vocabulary.
//!
//! Split out of `config/mod.rs` to keep that file under the size gate; re-exported
//! at `crate::config::{Legend, LegendEntry}`.

use std::collections::{HashSet, VecDeque};

use serde::Deserialize;

use crate::link::NoteRule;
use crate::problem::Problem;
use crate::suggest::Unknown;

/// The status legend: the legal status values, in lifecycle order.
///
/// Each entry gives a value its glyph, colour, label and `terminal` flag. An
/// empty legend (`status = []`) accepts any status. Omitting
/// `[[tasks.status]]` takes the shipped six-status legend.
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
    /// The status name, as written in a task's `status:` field.
    pub value: String,
    /// Optional decorative glyph (emoji or ASCII).
    pub glyph: Option<String>,
    /// Optional colour: a named ANSI colour or `#RRGGBB`.
    pub color: Option<String>,
    /// Optional display label; falls back to `value`.
    pub label: Option<String>,
    /// Whether this status ends a task. The `terminal` filter matches tasks in
    /// it, and `mt` does not check their tags or estimate units.
    pub terminal: bool,
    /// Statuses a task may move to from this one. While no entry sets `to`,
    /// any move is allowed. Once any entry sets it, `mt flip` refuses a move
    /// the legend does not list, and an entry with no `to` allows only a
    /// same-status re-stamp. A terminal status reopens by listing the status
    /// to reopen into.
    pub to: Vec<String>,
    /// Whether a move into this status needs a note: `required` or
    /// `optional` (the default). With `required`, `mt flip` refuses the move
    /// without `-m`, and `mt check` flags a task here with no note.
    /// `forbidden` is accepted but not enforced for a status.
    pub note: NoteRule,
    /// Whether a task must be unblocked to enter this status. `mt flip`
    /// refuses the move while a direct blocking target is unfinished, and
    /// `--force` overrides. On a terminal status, every child must be
    /// finished too. `mt check` warns about a blocked task already here. Off
    /// by default. Set it where work starts or finishes, and leave it off
    /// statuses that blocked work must still reach, such as `cancelled`.
    pub unblocked: bool,
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

    /// Every entry's value, in legend order.
    pub(crate) fn values(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.value.as_str()).collect()
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

    /// Whether entering `value` requires an unblocked task (its entry sets
    /// `unblocked = true`). False for an unknown value.
    #[must_use]
    pub fn requires_unblocked(&self, value: &str) -> bool {
        self.entry(value).is_some_and(|entry| entry.unblocked)
    }

    /// Whether entering `value` requires every child finished: its entry is
    /// both `unblocked` and `terminal`. The one knob covers both refusals —
    /// a status that needs the task's dependencies done and that ends the task
    /// also needs its children done — while an `unblocked` status work merely
    /// starts in (`wip`) leaves a parent free to start over open children.
    #[must_use]
    pub fn requires_finished_children(&self, value: &str) -> bool {
        self.entry(value)
            .is_some_and(|entry| entry.unblocked && entry.terminal)
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
    pub fn transition_problems(&self) -> Vec<Problem> {
        let mut problems = Vec::new();
        let known = self.values();
        for (index, entry) in self.entries.iter().enumerate() {
            for next in &entry.to {
                if !self.contains(next) {
                    problems.push(
                        Problem::new(format!(
                            "tasks.status[{index}].to: unknown status: {}",
                            super::quote(next)
                        ))
                        .with(Unknown::near(next, &known)),
                    );
                }
            }
        }
        problems
    }
}

#[cfg(test)]
#[path = "status_tests.rs"]
mod tests;
