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

use crate::cmd::link::edit::{EdgeSpec, Mode, links_edit};
use crate::cmd::link::input::Entry;
use crate::diag::{Command, Diag, Hint, Note};
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
            plan.problems.push(err.into());
            plan.failed += 1;
            return;
        }
    };
    let edges = settle_targets(project, &task, files, &entry.edges);
    let refusals = check_edges(project, &task, files, (&edges, &entry.edges));
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

/// Settle each edge's target into the literal string that goes into the file.
///
/// A `local-task-path` kind's target is a task file. A bare stem (no `/`) names
/// a task anywhere in the project, as `mt link --help` promises: a sibling
/// carrying it wins and is written `<stem>.typ`, else the one loaded task with
/// that stem is written relative to the linking task (`../cli/other.typ`),
/// since `resolve_link_path` reads every written target that way. A stem no
/// task or several tasks carry stays `<stem>.typ` for [`target_problem`] to
/// refuse. A path survives verbatim. A kind pointing *outside* the backlog is
/// left alone entirely: appending `.typ` to `https://…/pull/42` would corrupt
/// the one thing that edge records.
///
/// This runs before the checks, so every later stage — resolution, rendering,
/// the report — speaks about the string that was actually written.
fn settle_targets(
    project: &Project,
    task: &Task,
    files: &HashSet<&Path>,
    edges: &[EdgeSpec],
) -> Vec<EdgeSpec> {
    let links = &project.config.tasks.links;
    edges
        .iter()
        .map(|edge| {
            let target = if kind_targets_tasks(&edge.kind, links) {
                settle_task_target(project, task, files, &edge.target)
            } else {
                edge.target.clone()
            };
            EdgeSpec {
                target,
                ..edge.clone()
            }
        })
        .collect()
}

/// One task-targeting literal, per [`settle_targets`].
fn settle_task_target(
    project: &Project,
    task: &Task,
    files: &HashSet<&Path>,
    target: &str,
) -> String {
    if target.contains('/') {
        return target.to_owned();
    }
    let sibling = if target.ends_with(".typ") {
        target.to_owned()
    } else {
        format!("{target}.typ")
    };
    let resolves =
        resolve_link_path(&task.file, &sibling).is_some_and(|p| files.contains(p.as_path()));
    match stem_candidates(project, task, target).as_slice() {
        [only] if !resolves => only.clone(),
        _ => sibling,
    }
}

/// Every loaded task whose stem is `target`'s, as a literal relative to
/// `task`'s folder.
fn stem_candidates(project: &Project, task: &Task, target: &str) -> Vec<String> {
    let stem = target.strip_suffix(".typ").unwrap_or(target);
    let from = task.file.parent().unwrap_or(Path::new(""));
    project
        .outcome
        .tasks
        .iter()
        .filter(|other| other.id == stem)
        .map(|other| {
            util::relative_target(from, &other.file)
                .display()
                .to_string()
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
) -> Result<Option<Rewritten>, Diag> {
    let source = std::fs::read_to_string(abs).map_err(|err| Diag::cannot_read(abs, &err))?;
    let Some(edit) = links_edit(&source, &task.links, edges, mode)
        .map_err(|err| Diag::from(err).at_file(abs))?
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

/// Every reason `edges` cannot be written onto `task`, collected rather than
/// short-circuited so one run reports all of them instead of one per re-run.
/// `edges` is the settled and the typed edge list, in step: a message names the
/// target as the user typed it.
fn check_edges(
    project: &Project,
    task: &Task,
    files: &HashSet<&Path>,
    (edges, typed): (&[EdgeSpec], &[EdgeSpec]),
) -> Vec<Diag> {
    let links = &project.config.tasks.links;
    let mut refusals = Vec::new();
    for (edge, typed) in edges.iter().zip(typed) {
        let Some(rule) = resolve_kind(&edge.kind, links) else {
            refusals.push(unknown_kind(&edge.kind, links, task));
            continue;
        };
        if let Some(why) = note_problem(edge, typed, rule.note) {
            refusals.push(why.at_file(&task.file));
        }
        if kind_targets_tasks(&edge.kind, links)
            && let Some(why) = target_problem(project, task, (edge, typed), files)
        {
            refusals.push(why);
        }
    }
    refusals
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
fn note_problem(edge: &EdgeSpec, typed: &EdgeSpec, rule: NoteRule) -> Option<Diag> {
    let note = edge
        .note
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty());
    match rule {
        NoteRule::Required if note.is_none() => Some(
            Diag::refused("edge requires a note")
                .value(typed.target.as_str())
                .context("of kind", edge.kind.as_str())
                .hint(Hint::new(
                    "pass one with -m, or a note: field on the target in the map",
                )),
        ),
        NoteRule::Forbidden if note.is_some() => Some(
            Diag::refused("edge must not carry a note")
                .value(typed.target.as_str())
                .context("of kind", edge.kind.as_str()),
        ),
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
    (edge, typed): (&EdgeSpec, &EdgeSpec),
    files: &HashSet<&Path>,
) -> Option<Diag> {
    let resolved = resolve_link_path(&task.file, &edge.target);
    if resolved.as_deref().is_some_and(|path| files.contains(path)) {
        return None;
    }
    let candidates = stem_candidates(project, task, &edge.target);
    if !edge.target.contains('/') && candidates.len() > 1 {
        return Some(
            candidates.iter().fold(
                Diag::failed("edge target is ambiguous")
                    .at_file(&task.file)
                    .value(typed.target.as_str())
                    .context("of kind", edge.kind.as_str())
                    .hint(Hint::new("pass the path of the one you mean")),
                |diag, candidate| diag.note(Note::new("candidate").name(candidate.clone())),
            ),
        );
    }
    let ids: Vec<&str> = project
        .outcome
        .tasks
        .iter()
        .map(|task| task.id.as_str())
        .collect();
    let diag = Diag::failed("edge target names no task")
        .at_file(&task.file)
        .value(typed.target.as_str())
        .context("of kind", edge.kind.as_str());
    Some(
        match suggest::nearest(&mindtape::normalize_link(&edge.target), &ids) {
            Some(near) => diag.hint(Hint::did_you_mean(near)),
            None => diag.hint(Hint::run(Command::mt().word("ls"), "to see your tasks")),
        },
    )
}

#[cfg(test)]
#[path = "plan_tests.rs"]
mod tests;
