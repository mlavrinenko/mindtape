//! The guards one `flip` clears before its write, split out of `cmd/flip/mod.rs`
//! (file-size gate): the declared transition graph, `unblocked` blockers,
//! unfinished children, and the target's note requirement. Each returns
//! `Err(message)` naming what refused the move; every caller shares the
//! `--force` escape hatch.

use mindtape::config::Legend;

/// Guard one flip against the declared transition graph. `Ok(())` when the move
/// is legal or unconstrained; `Err(message)` names the illegal edge and the
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
) -> std::result::Result<(), String> {
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
    let listing = if allowed.is_empty() {
        format!("`{}` is terminal here (no declared transitions)", from.name)
    } else {
        format!("allowed from `{}`: {}", from.name, allowed.join(", "))
    };
    Err(format!(
        "illegal transition `{}` → `{to}`; {listing} (use --force to override)",
        from.name
    ))
}

/// Guard one flip against the target status's `unblocked = true`: refused
/// while `blockers` (the task's unfinished blocking-kind targets) is non-empty,
/// naming each one. Shares the transition guard's `--force` escape hatch.
pub(super) fn check_unblocked(
    legend: &Legend,
    to: &str,
    blockers: &[mindtape::Blocker],
) -> std::result::Result<(), String> {
    if blockers.is_empty() || !legend.requires_unblocked(to) {
        return Ok(());
    }
    Err(format!(
        "{} (use --force to override)",
        mindtape::unblocked_violation(to, blockers)
    ))
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
) -> std::result::Result<(), String> {
    if !config.tasks.status.requires_finished_children(to) {
        return Ok(());
    }
    let open = mindtape::unfinished_children(world, task, config);
    if open.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{} (use --force to override)",
        mindtape::children_violation(to, &open)
    ))
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
) -> std::result::Result<(), String> {
    let required = legend
        .entry(to)
        .is_some_and(mindtape::config::LegendEntry::note_required);
    if !required || note.map(str::trim).is_some_and(|n| !n.is_empty()) {
        return Ok(());
    }
    Err(format!("`{to}` requires a note; supply one with -m/--note"))
}
