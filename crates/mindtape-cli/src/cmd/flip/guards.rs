//! The guards one `flip` clears before its write, split out of `cmd/flip/mod.rs`
//! (file-size gate): the declared transition graph, `unblocked` blockers,
//! unfinished children, and the target's note requirement. Each returns
//! `Err(Diag)` naming what refused the move; every caller shares the
//! `--force` escape hatch.

use mindtape::config::Legend;

use crate::diag::{Diag, Hint, Note};

/// The way past a guard.
fn force_hint() -> Hint {
    Hint::new("or pass --force to override")
}

/// `a (proposed), b (wip)`: the tasks a guard names; one with no known status
/// reads `unknown`.
fn targets(blockers: &[mindtape::Blocker]) -> String {
    let list: Vec<String> = blockers
        .iter()
        .map(|b| format!("{} ({})", b.stem, b.status.as_deref().unwrap_or("unknown")))
        .collect();
    list.join(", ")
}

/// Guard one flip against the declared transition graph. `Ok(())` when the move
/// is legal or unconstrained; `Err(Diag)` names the illegal edge and the
/// allowed next states.
///
/// Semantics (all opt-in behind [`Legend::transitions_declared`]):
/// - no graph declared → unconstrained, always `Ok`;
/// - unset current status → exempt, the graph governs status-to-status only;
/// - same-status re-stamp (`from == to`) → always allowed;
/// - otherwise the edge must be in the current status's `to` list.
pub(super) fn check_transition(
    legend: &Legend,
    from: Option<&mindtape::Status>,
    to: &str,
) -> Result<(), Diag> {
    if !legend.transitions_declared() {
        return Ok(());
    }
    let Some(from) = from else {
        return Ok(());
    };
    if from.name == to || legend.can_transition(&from.name, to) {
        return Ok(());
    }
    let allowed = legend.allowed_next(&from.name);
    let diag = Diag::refused("illegal transition").value(format!("{} → {to}", from.name));
    let diag = if allowed.is_empty() {
        diag.note(Note::new("no transitions are declared from").value(&from.name))
    } else {
        diag.note(Note::new("allowed next").name(format!("{} → {}", from.name, allowed.join(", "))))
    };
    Err(diag.hint(force_hint()))
}

/// Guard one flip against the target status's `unblocked = true`: refused
/// while `blockers` (the task's unfinished blocking-kind targets) is non-empty,
/// naming each one. Shares the transition guard's `--force` escape hatch.
pub(super) fn check_unblocked(
    legend: &Legend,
    to: &str,
    blockers: &[mindtape::Blocker],
) -> Result<(), Diag> {
    if blockers.is_empty() || !legend.requires_unblocked(to) {
        return Ok(());
    }
    Err(Diag::refused("status requires an unblocked task")
        .value(to)
        .note(Note::new("waits on").name(targets(blockers)))
        .hint(force_hint()))
}

/// Guard one flip against a status that requires finished children (both
/// `unblocked` and `terminal`): refused while any task naming `task` as its
/// `parent` is unfinished within `world`, naming each one. Shares the
/// transition guard's `--force` escape hatch.
pub(super) fn check_children(
    world: &[mindtape::Task],
    task: &mindtape::Task,
    to: &str,
    config: &mindtape::Config,
) -> Result<(), Diag> {
    if !config.tasks.status.requires_finished_children(to) {
        return Ok(());
    }
    let open = mindtape::unfinished_children(world, task, config);
    if open.is_empty() {
        return Ok(());
    }
    Err(Diag::refused("status requires finished children")
        .value(to)
        .note(Note::new("unfinished").name(targets(&open)))
        .hint(force_hint()))
}

/// Guard one flip against the target status's note requirement (`note =
/// "required"` on its `[[tasks.status]]` entry — see
/// `mindtape::config::LegendEntry::note_required`, the same `NoteRule`
/// primitive typed `links:` edges use). `Ok(())` when the target carries no
/// requirement, or a non-blank note was supplied; `Err(message)` names the
/// status and points at the flag that supplies one.
pub(super) fn check_note_required(
    legend: &Legend,
    to: &str,
    note: Option<&str>,
) -> Result<(), Diag> {
    let required = legend
        .entry(to)
        .is_some_and(mindtape::config::LegendEntry::note_required);
    if !required || note.map(str::trim).is_some_and(|n| !n.is_empty()) {
        return Ok(());
    }
    Err(Diag::refused("status requires a note")
        .value(to)
        .hint(Hint::new("pass one with -m/--note")))
}
