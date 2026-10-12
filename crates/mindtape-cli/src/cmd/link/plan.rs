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
//! `mt add`'s pre-write validation follows. A target resolves through
//! [`Refs::target`], the reader every `<task>` argument shares.

use std::path::{Path, PathBuf};

use mindtape::link::{NoteRule, kind_names, kind_targets_tasks, resolve_kind};
use mindtape::{Task, suggest};

use crate::cmd::link::edit::{EdgeSpec, Mode, links_edit};
use crate::cmd::link::input::Entry;
use crate::diag::{Diag, Hint, Note};
use crate::util::{Project, Refs};

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
    /// Per-entry failures, each naming the entry it belongs to.
    pub problems: Vec<Diag>,
    /// How many entries failed, which can be fewer than `problems` when one
    /// entry has several.
    pub failed: usize,
}

/// Stage every entry against `project`. Never fails as a whole: an entry that
/// cannot be resolved, checked or rewritten becomes a [`Plan::problems`] line
/// and the rest are staged regardless.
#[must_use]
pub fn build(project: &Project, entries: &[Entry], mode: Mode) -> Plan {
    let mut plan = Plan::default();
    let refs = match Refs::new(project) {
        Ok(refs) => refs,
        Err(err) => {
            plan.problems.push(err);
            plan.failed = entries.len();
            return plan;
        }
    };
    for entry in entries {
        stage(project, &refs, entry, mode, &mut plan);
    }
    plan
}

/// Stage one entry, routing whatever it fails on into `plan.problems`.
fn stage(project: &Project, refs: &Refs<'_>, entry: &Entry, mode: Mode, plan: &mut Plan) {
    let (task, abs) = match refs.task(&entry.stem) {
        Ok(found) => (found.task.clone(), found.path),
        Err(err) => {
            plan.problems.push(err.into());
            plan.failed += 1;
            return;
        }
    };
    let (edges, refusals) = check_edges(project, refs, &task, &entry.edges);
    if !refusals.is_empty() {
        plan.problems.extend(refusals);
        plan.failed += 1;
        return;
    }
    match rewrite(&abs, &task, &edges, mode) {
        Err(err) => {
            plan.problems.push(err);
            plan.failed += 1;
        }
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
) -> Result<Option<Rewritten>, Diag> {
    let source = std::fs::read_to_string(abs).map_err(|err| Diag::cannot_read(abs, &err))?;
    let Some(edit) =
        links_edit(&source, &task.links, edges, mode).map_err(|err| err.at_file(abs))?
    else {
        return Ok(None);
    };
    let updated = typst_edit::apply(&source, vec![edit.edit]).map_err(|err| {
        Diag::failed("can't rewrite links:")
            .at_file(abs)
            .cause(&err)
    })?;
    Ok(Some(Rewritten {
        updated,
        added: edit.added,
        removed: edit.removed,
    }))
}

/// Check `typed` for `task` and settle each target into the literal the file
/// gets, collecting every refusal rather than short-circuiting, so one run
/// reports all of them instead of one per re-run.
///
/// A `local-task-path` kind's target is a task, written as the path from the
/// linking file to it, whether it was typed as a path, a stem or a stem hash.
/// A kind pointing *outside* the backlog keeps its target verbatim: appending
/// `.typ` to `https://…/pull/42` would corrupt the one thing that edge records.
fn check_edges(
    project: &Project,
    refs: &Refs<'_>,
    task: &Task,
    typed: &[EdgeSpec],
) -> (Vec<EdgeSpec>, Vec<Diag>) {
    let links = &project.config.tasks.links;
    let mut edges = Vec::with_capacity(typed.len());
    let mut refusals = Vec::new();
    for edge in typed {
        let Some(rule) = resolve_kind(&edge.kind, links) else {
            refusals.push(unknown_kind(&edge.kind, links, task));
            continue;
        };
        if let Some(why) = note_problem(edge, rule.note) {
            refusals.push(why.at_file(&task.file));
        }
        if !kind_targets_tasks(&edge.kind, links) {
            edges.push(edge.clone());
            continue;
        }
        match refs.target(task, &edge.target) {
            Ok(target) => edges.push(EdgeSpec {
                target,
                ..edge.clone()
            }),
            Err(why) => refusals.push(
                Diag::from(why)
                    .at_file(&task.file)
                    .context("of kind", edge.kind.as_str()),
            ),
        }
    }
    (edges, refusals)
}

/// The unknown-kind refusal: what was written, the nearest configured name
/// when one clears the similarity floor, and what is configured.
fn unknown_kind(kind: &str, links: &mindtape::link::LinksConfig, task: &Task) -> Diag {
    let names = kind_names(links);
    let diag = Diag::usage("unknown link kind")
        .at_file(&task.file)
        .value(kind);
    let diag = match suggest::nearest(kind, &names) {
        Some(near) => diag.hint(Hint::did_you_mean(near)),
        None => diag,
    };
    diag.note(Note::new("configured kinds").name(names.join(", ")))
}

/// The note-rule refusal for one edge, or `None` when it satisfies the kind's
/// rule. Named per edge, not per task: a bulk map declaring twenty `related`
/// edges has to say which ones are missing a reason.
fn note_problem(edge: &EdgeSpec, rule: NoteRule) -> Option<Diag> {
    let note = edge
        .note
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty());
    match rule {
        NoteRule::Required if note.is_none() => Some(
            Diag::refused("edge requires a note")
                .value(edge.target.as_str())
                .context("of kind", edge.kind.as_str())
                .hint(Hint::new(
                    "pass one with -m, or a note: field on the target in the map",
                )),
        ),
        NoteRule::Forbidden if note.is_some() => Some(
            Diag::refused("edge must not carry a note")
                .value(edge.target.as_str())
                .context("of kind", edge.kind.as_str()),
        ),
        NoteRule::Required | NoteRule::Optional | NoteRule::Forbidden => None,
    }
}

#[cfg(test)]
#[path = "plan_tests.rs"]
mod tests;
