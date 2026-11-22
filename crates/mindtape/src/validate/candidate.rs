//! Validate a rendered task *before* a writer writes it
//! (`truth/tasks/2026-09/mt-add-writes-an-invalid-task-before-it-validates-one.typ`).
//!
//! [`load_project`](crate::load_project) answers "what do the files on disk
//! say". This module answers "what would they say if this one more file
//! landed here", without landing it. A writer (`mt add`) renders into a
//! buffer, asks [`check_candidate`], and only then commits the buffer to its
//! path. A rejected write leaves the project as it was.
//!
//! [`check_candidates`] asks the same question of many buffers at once, for
//! the writers whose unit of work is a batch (`mt import`, `mt link`,
//! `mt recur --advance`). It is not a convenience wrapper over a loop: the
//! rules that span tasks have no answer for one member of a batch taken alone.
//!
//! The check evaluates the candidate exactly where it would live.
//! [`Candidate::rel`] is the file id Typst resolves relative imports against,
//! and the loader reads the folder-derived tags and nesting depth off that
//! same path. What makes this possible without a file is
//! [`World::with_main_source`](typst_world::World::with_main_source), which
//! holds the main source in memory and reads every import from disk.

use std::collections::HashSet;
use std::path::Path;

use typst_harvest::{HVal, HarvestWorld, harvest};
use typst_world::project_file_id;

use crate::body::{BodyRules, check_body_rules};
use crate::config::Config;
use crate::link::acyclic_kinds;
use crate::loader::{LoadError, StatePackages};
use crate::model::Task;
use crate::validate::Violation;

/// A task source that does not exist on disk, and the project-relative path a
/// writer would give it.
#[derive(Debug, Clone, Copy)]
pub struct Candidate<'a> {
    /// Destination path, relative to the project root.
    pub rel: &'a Path,
    /// The rendered task source, exactly as the writer would write it.
    pub source: &'a str,
}

/// A candidate that evaluated, and what the project makes of it.
#[derive(Debug, Clone)]
pub struct Checked {
    /// The task the candidate would load as, once written.
    pub task: Task,
    /// The violations naming that task, with every candidate in the same batch
    /// already standing beside the rest of the project.
    pub violations: Vec<Violation>,
    /// The breaches the batch opens or closes on a task it never writes, a
    /// link cycle or a rule reading the whole set. They name the other task
    /// and are the same for every candidate of the batch. They refuse the
    /// write as a candidate's own do. A caller prints them once per batch.
    pub elsewhere: Vec<Violation>,
}

/// Validate `candidate` against `existing` as if a writer had already put it at
/// [`Candidate::rel`], returning the violations that name the candidate itself,
/// then those the write raises on another task ([`Checked::elsewhere`]).
///
/// `existing` holds the project's already-loaded tasks, so no second load
/// happens here. The check replaces any task already at `rel`, which makes a
/// rewrite count as a rewrite rather than as a duplicate id.
///
/// `body_rules` holds `mt check`'s own header ([`crate::body::gate`]) and
/// content ([`crate::body::content_gate`]) rules. The caller compiles them once
/// and this function runs them over `candidate.source` directly, the
/// candidate's own text rather than a loaded file's, beside
/// [`crate::validate::validate`]. This function also refuses a body-rule breach
/// that `mt check` would report, before anything reaches disk.
///
/// # Errors
/// Returns a [`LoadError`] when the source fails to evaluate or harvests no
/// `mindtape.task` marker. Both are failures of the source, not of the project.
pub fn check_candidate(
    root: &Path,
    config: &Config,
    existing: &[Task],
    candidate: Candidate<'_>,
    body_rules: BodyRules<'_>,
) -> Result<Vec<Violation>, LoadError> {
    check_candidates(
        root,
        config,
        existing,
        std::slice::from_ref(&candidate),
        body_rules,
    )
    .into_iter()
    .next()
    // One answer per candidate, and exactly one candidate was asked about;
    // the fallback is unreachable rather than a policy.
    .map_or_else(
        || Ok(Vec::new()),
        |result| {
            result.map(|mut ok| {
                ok.violations.append(&mut ok.elsewhere);
                ok.violations
            })
        },
    )
}

/// Validate a whole batch of candidates as if a writer had written every one of
/// them, answering one entry per candidate in input order.
///
/// The check runs over a batch together rather than one at a time. The rules
/// that span tasks cannot answer for one candidate without the others: id
/// uniqueness, edge resolution and cardinality, cycles, and the dependency
/// ranks a `required` filter reads. `mt link --apply` merging an adjacency map
/// and `mt recur --advance` chaining occurrences that name their predecessor
/// are both batches whose members only make sense as a set.
///
/// Failure stays per entry, though. A candidate that does not evaluate gets
/// its own `Err` and simply never joins the set. One unparseable entry does
/// not cost the rest their verdict.
///
/// The check replaces every task already sitting at one of the candidates'
/// paths. A batch of rewrites then counts as rewrites rather than as duplicate
/// ids.
///
/// `body_rules` holds `mt check`'s own header and content rules
/// ([`crate::body`]). The caller compiles them once for the whole batch, and
/// this function runs them over each candidate's own `source`, never over an
/// existing task's. A body-rule breach already on disk cannot leak onto an
/// unrelated candidate.
#[must_use]
pub fn check_candidates(
    root: &Path,
    config: &Config,
    existing: &[Task],
    candidates: &[Candidate<'_>],
    body_rules: BodyRules<'_>,
) -> Vec<Result<Checked, LoadError>> {
    let mut harvested: Vec<Result<Task, LoadError>> = candidates
        .iter()
        .map(|candidate| harvest_candidate(root, config, *candidate))
        .collect();

    let replaced: HashSet<&Path> = candidates.iter().map(|candidate| candidate.rel).collect();
    let mut tasks: Vec<Task> = existing
        .iter()
        .filter(|task| !replaced.contains(task.file.as_path()))
        .cloned()
        .collect();
    tasks.extend(
        harvested
            .iter()
            .filter_map(|result| result.as_ref().ok())
            .cloned(),
    );
    tasks.sort_by(|left, right| left.id.cmp(&right.id));
    crate::graph::dep_rank::annotate(&mut tasks, config);
    crate::waits::annotate(&mut tasks, config, root);
    crate::estimate_queries::annotate(&mut tasks, config, root);
    crate::collect::annotate(&mut tasks, config);
    for result in &mut harvested {
        if let Ok(task) = result
            && let Some(annotated) = tasks.iter().find(|candidate| candidate.file == task.file)
        {
            task.derived.clone_from(&annotated.derived);
        }
    }
    let violations = crate::validate::validate(&tasks, config);
    let ids: HashSet<&str> = harvested
        .iter()
        .filter_map(|result| result.as_ref().ok())
        .map(|task| task.id.as_str())
        .collect();
    let closed = elsewhere_violations(existing, config, &ids, &violations);

    harvested
        .into_iter()
        .zip(candidates)
        .map(|(result, candidate)| {
            result.map(|task| {
                let mut task_violations: Vec<Violation> = violations
                    .iter()
                    .filter(|violation| violation.task_id == task.id)
                    .cloned()
                    .collect();
                task_violations.extend(check_body_rules(
                    &task,
                    candidate.source,
                    body_rules,
                    config,
                ));
                Checked {
                    violations: task_violations,
                    elsewhere: closed.clone(),
                    task,
                }
            })
        })
        .collect()
}

/// Whether `violation` is a breach only the set around a task can cause or
/// cure. Such a breach is a link cycle on an acyclic kind, or a rule whose
/// expression reads the whole set (`count(...)`, `ready`, `topo`). It is also
/// an edge to a task that is not in the set. A task checked apart from its
/// backlog drops these, as the task cannot fix them. It also drops a rule
/// skipped for lack of git, a breach of the config reported once at its entry.
#[must_use]
pub fn reads_set_breach(config: &Config, violation: &Violation) -> bool {
    crate::validate::links::is_unresolved(violation)
        || violation.kind == crate::validate::violation::ViolationKind::SkippedRule
        || acyclic_kinds(&config.tasks.links).contains(&violation.field.as_str())
        || crate::rules::set_labels(config).contains(&violation.field)
}

/// The violations the batch opens or closes elsewhere that `validate` names on
/// a task outside it. Such a violation is a link cycle, or a `[[tasks.rules]]`
/// breach whose `assert` reads the whole set (`count(...)`, or the `links:`
/// graph through `ready`/`blocked`/`blocks`/`topo`).
///
/// Both share the same shape: a fact about a task outside the batch that only
/// changed because a candidate did. Both go to every candidate's
/// [`Checked::elsewhere`], worded as `mt check` words them and naming the task
/// they are on. A cycle reports on its first member, which need not be a task
/// the batch writes. Filtering by candidate id alone drops it, and the write
/// then lands a graph `mt check` rejects on a neighbour. A WIP-cap rule the
/// batch pushes over its cap reports on the task(s) already occupying it, which
/// the batch never touched. Either kind the project already carried before the
/// batch is not the batch's doing and never counts.
fn elsewhere_violations(
    existing: &[Task],
    config: &Config,
    ids: &HashSet<&str>,
    after: &[Violation],
) -> Vec<Violation> {
    let acyclic = acyclic_kinds(&config.tasks.links);
    let rule_labels = crate::rules::labels(config);
    let elsewhere: Vec<&Violation> = after
        .iter()
        .filter(|violation| {
            !ids.contains(violation.task_id.as_str())
                && (acyclic.contains(&violation.field.as_str())
                    || rule_labels.contains(&violation.field))
        })
        .collect();
    if elsewhere.is_empty() {
        return Vec::new();
    }
    let mut before_tasks = existing.to_vec();
    before_tasks.sort_by(|left, right| left.id.cmp(&right.id));
    let before = crate::validate::validate(&before_tasks, config);
    elsewhere
        .into_iter()
        .filter(|violation| {
            !before.iter().any(|old| {
                old.task_id == violation.task_id
                    && old.field == violation.field
                    && old.message == violation.message
            })
        })
        .cloned()
        .collect()
}

/// Evaluate `candidate`'s source at its destination path and build the [`Task`]
/// it would load as: identity from the stem, folder-derived tags and depth from
/// the path, body from the source.
///
/// # Errors
/// Returns a [`LoadError`] when the world fails to build, evaluation fails, or
/// the source carries no `mindtape.task` marker.
pub fn harvest_candidate(
    root: &Path,
    config: &Config,
    candidate: Candidate<'_>,
) -> Result<Task, LoadError> {
    harvest_source(root, config, candidate)?.ok_or_else(|| LoadError {
        file: candidate.rel.to_path_buf(),
        message: "source harvested no task marker".to_owned(),
        ..LoadError::default()
    })
}

/// [`harvest_candidate`], reading a source with no task marker as no task
/// rather than an error, the way the loader reads a `.typ` beside the tasks
/// that is not one. What a commit state's version of a file loads as
/// ([`crate::commit::tasks`]).
pub(crate) fn harvest_source(
    root: &Path,
    config: &Config,
    candidate: Candidate<'_>,
) -> Result<Option<Task>, LoadError> {
    harvest_source_with(root, config, candidate, None)
}

/// [`harvest_source`] with a commit state's own package files tried first. A
/// past tree's imports then resolve against that tree's `lib/`.
pub(crate) fn harvest_source_with(
    root: &Path,
    config: &Config,
    candidate: Candidate<'_>,
    packages: Option<StatePackages>,
) -> Result<Option<Task>, LoadError> {
    Ok(
        eval_marker(root, config, candidate, packages)?.map(|marker| {
            crate::loader::task_at(
                root,
                config,
                candidate.rel,
                crate::loader::BodyMarkers {
                    task: &marker,
                    body: &[],
                },
                candidate.source,
            )
        }),
    )
}

/// Evaluate the candidate source and return its first `mindtape.task` marker.
fn eval_marker(
    root: &Path,
    config: &Config,
    candidate: Candidate<'_>,
    packages: Option<StatePackages>,
) -> Result<Option<HVal>, LoadError> {
    let fail = |message: String| LoadError {
        file: candidate.rel.to_path_buf(),
        message,
        ..LoadError::default()
    };
    let root_abs = crate::canonical(root).unwrap_or_else(|_| root.to_path_buf());
    let id = project_file_id(&root_abs, &root_abs.join(candidate.rel))
        .map_err(|err| fail(err.to_string()))?;
    let world = HarvestWorld::with_main_source(id, root_abs, candidate.source);
    let overrides = crate::loader::package_overrides(root, config);
    let world = match packages {
        Some(packages) => crate::loader::with_packages_and(world, &overrides, packages),
        None => crate::loader::with_packages(world, &overrides),
    };

    let harvested = harvest(&world)
        .map_err(|err| LoadError::harvest(candidate.rel.to_path_buf(), &err, candidate.source))?;
    Ok(harvested
        .with_marker("mindtape.task")
        .next()
        .map(|found| found.value.clone()))
}

#[cfg(test)]
#[path = "candidate_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "candidate_tests_body.rs"]
mod tests_body;
