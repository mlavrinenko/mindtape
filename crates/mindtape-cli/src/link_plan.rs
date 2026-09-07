//! `mt link`'s plan: resolve every entry, check every edge against the
//! project's kind registry, and stage each rewrite in memory.
//!
//! Nothing is written here — the whole batch is computed first, `rm`-style, so
//! `--dry-run` reports exactly what a real run would do rather than a
//! separately-derived guess at it. A per-entry failure stays with that entry:
//! the rest of the batch still lands, which is what makes a bulk `--apply`
//! usable over a map the caller assembled from another tool.
//!
//! What is checked here rather than left to the post-write `mt check`: the
//! kind, the note rule, and target resolution — the three that make a written
//! file invalid rather than merely different. Refusing them before the write
//! means a rejected edge leaves nothing on disk to repair, the same discipline
//! `mt add`'s pre-write validation follows.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use mindtape::link::{NoteRule, kind_names, kind_targets_tasks, resolve_kind};
use mindtape::model::resolve_link_path;
use mindtape::{Task, suggest};

use crate::link_edit::{EdgeSpec, Mode, links_edit};
use crate::link_input::Entry;
use crate::util::{self, Project};

/// One task's staged rewrite, ready to write or to report.
pub struct Staged {
    /// The linking task's id, as it will be reported.
    pub stem: String,
    /// Absolute path to its file.
    pub abs: PathBuf,
    /// The loaded task, for the post-write revalidation pass.
    pub task: Task,
    /// The rewritten source, canonicalized on write, not here.
    pub updated: String,
    /// Edges this rewrite adds.
    pub added: Vec<EdgeSpec>,
    /// Edges it drops (`--replace` only).
    pub removed: usize,
}

/// What a whole `mt link` run will do, before any of it happens.
#[derive(Default)]
pub struct Plan {
    /// Files to rewrite.
    pub staged: Vec<Staged>,
    /// Entries whose task already says exactly this — reported, not written.
    pub unchanged: Vec<String>,
    /// Per-entry failures, already prefixed with the entry they belong to.
    pub problems: Vec<String>,
}

/// Stage every entry against `project`. Never fails as a whole: an entry that
/// cannot be resolved, checked or rewritten becomes a [`Plan::problems`] line
/// and the rest are staged regardless.
#[must_use]
pub fn build(project: &Project, entries: &[Entry], mode: Mode) -> Plan {
    let files: HashSet<&Path> = project
        .outcome
        .tasks
        .iter()
        .map(|task| task.file.as_path())
        .collect();
    let mut plan = Plan::default();
    for entry in entries {
        stage(project, &files, entry, mode, &mut plan);
    }
    plan
}

/// Stage one entry, routing whatever it fails on into `plan.problems`.
fn stage(project: &Project, files: &HashSet<&Path>, entry: &Entry, mode: Mode, plan: &mut Plan) {
    let stem = entry.stem.as_str();
    let (task, abs) = match util::resolve_task(project, stem) {
        Ok(resolved) => resolved,
        Err(err) => {
            plan.problems.push(format!("{stem}: {err:#}"));
            return;
        }
    };
    let edges = settle_targets(project, &entry.edges);
    let refusals = check_edges(project, &task, files, &edges);
    if !refusals.is_empty() {
        plan.problems
            .extend(refusals.into_iter().map(|why| format!("{stem}: {why}")));
        return;
    }
    match rewrite(&abs, &task, &edges, mode) {
        Err(err) => plan.problems.push(format!("{stem}: {err:#}")),
        Ok(None) => plan.unchanged.push(task.id.clone()),
        Ok(Some(rewritten)) => plan.staged.push(Staged {
            stem: task.id.clone(),
            abs,
            task,
            updated: rewritten.updated,
            added: rewritten.added,
            removed: rewritten.removed,
        }),
    }
}

/// Settle each edge's target into the literal string that goes into the file.
///
/// A `local-task-path` kind's target is a task file, so a bare stem gains the
/// `.typ` the DSL's targets carry (`other` -> `other.typ`) while a path or an
/// already-suffixed stem survives verbatim — `resolve_link_path` reads a target
/// relative to the linking task, so `../sub/x.typ` has to stay exactly that.
/// A kind pointing *outside* the backlog is left alone entirely: appending
/// `.typ` to `https://…/pull/42` would corrupt the one thing that edge records.
///
/// This runs before the checks, so every later stage — resolution, rendering,
/// the report — speaks about the string that was actually written.
fn settle_targets(project: &Project, edges: &[EdgeSpec]) -> Vec<EdgeSpec> {
    let links = &project.config.tasks.links;
    edges
        .iter()
        .map(|edge| {
            let bare = kind_targets_tasks(&edge.kind, links) && !edge.target.ends_with(".typ");
            EdgeSpec {
                target: if bare {
                    format!("{}.typ", edge.target)
                } else {
                    edge.target.clone()
                },
                ..edge.clone()
            }
        })
        .collect()
}

/// One file's computed rewrite: the new source and what changed in it.
struct Rewritten {
    updated: String,
    added: Vec<EdgeSpec>,
    removed: usize,
}

/// Read `abs`, compute the `links:` rewrite, and splice it. `Ok(None)` when
/// the file already carries these edges.
fn rewrite(
    abs: &Path,
    task: &Task,
    edges: &[EdgeSpec],
    mode: Mode,
) -> anyhow::Result<Option<Rewritten>> {
    use anyhow::Context as _;
    let source = std::fs::read_to_string(abs).with_context(|| format!("read {}", abs.display()))?;
    let Some(edit) = links_edit(&source, &task.links, edges, mode)? else {
        return Ok(None);
    };
    let updated = typst_edit::apply(&source, vec![edit.edit])
        .map_err(|err| anyhow::anyhow!("rewrite `links:`: {err}"))?;
    Ok(Some(Rewritten {
        updated,
        added: edit.added,
        removed: edit.removed,
    }))
}

/// Every reason `edges` cannot be written onto `task`, collected rather than
/// short-circuited so one run reports all of them instead of one per re-run.
fn check_edges(
    project: &Project,
    task: &Task,
    files: &HashSet<&Path>,
    edges: &[EdgeSpec],
) -> Vec<String> {
    let links = &project.config.tasks.links;
    let mut refusals = Vec::new();
    for edge in edges {
        let Some(rule) = resolve_kind(&edge.kind, links) else {
            refusals.push(unknown_kind(&edge.kind, links));
            continue;
        };
        if let Some(why) = note_problem(edge, rule.note) {
            refusals.push(why);
        }
        if kind_targets_tasks(&edge.kind, links)
            && let Some(why) = target_problem(project, task, edge, files)
        {
            refusals.push(why);
        }
    }
    refusals
}

/// The unknown-kind refusal: what was written, what is configured, and the
/// nearest configured name when one clears the similarity floor — the same
/// enumerate-then-suggest shape `mt flip`'s unknown-status error carries.
fn unknown_kind(kind: &str, links: &mindtape::link::LinksConfig) -> String {
    let names = kind_names(links);
    format!(
        "`{kind}` is not a configured link kind; configured: {}{}",
        names.join(", "),
        suggest::suggestion(kind, &names)
    )
}

/// The note-rule refusal for one edge, or `None` when it satisfies the kind's
/// rule. Named per edge, not per task: a bulk map declaring twenty `related`
/// edges has to say which ones are missing a reason.
fn note_problem(edge: &EdgeSpec, rule: NoteRule) -> Option<String> {
    let note = edge
        .note
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty());
    match rule {
        NoteRule::Required if note.is_none() => Some(format!(
            "`{}` edge to `{}` requires a note; supply one with -m/--note, \
             or a `note:` field on the target in the map",
            edge.kind, edge.target
        )),
        NoteRule::Forbidden if note.is_some() => Some(format!(
            "`{}` edge to `{}` must not carry a note",
            edge.kind, edge.target
        )),
        NoteRule::Required | NoteRule::Optional | NoteRule::Forbidden => None,
    }
}

/// The unresolved-target refusal for a task-targeting edge, or `None` when the
/// target names a loaded task. Resolution is relative to the linking task's own
/// directory, exactly as `mt check` reads the written edge back.
///
/// The nearest match alone, never the full enumeration: the candidate set here
/// is every task in the project, which a 2000-task backlog would print in full.
fn target_problem(
    project: &Project,
    task: &Task,
    edge: &EdgeSpec,
    files: &HashSet<&Path>,
) -> Option<String> {
    let resolved = resolve_link_path(&task.file, &edge.target);
    if resolved.as_deref().is_some_and(|path| files.contains(path)) {
        return None;
    }
    let ids: Vec<&str> = project
        .outcome
        .tasks
        .iter()
        .map(|task| task.id.as_str())
        .collect();
    Some(format!(
        "`{}` edge target `{}` names no task{}",
        edge.kind,
        edge.target,
        suggest::suggestion(&mindtape::normalize_link(&edge.target), &ids)
    ))
}

#[cfg(test)]
#[path = "link_plan_tests.rs"]
mod tests;
