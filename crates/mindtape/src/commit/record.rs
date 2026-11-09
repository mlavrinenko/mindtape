//! The commit record a `scope = "commit"` rule evaluates.

use std::path::Path;

use crate::commit::tree_io::project_rel;
use crate::config::Config;
use crate::model::Task;
use crate::problem::Problem;
use crate::suggest::Unknown;
use crate::task_source::TaskSource;

use super::trailer::{is_path_form, ref_text, value_stem};
use super::{ChangedFiles, CommitMessage, RefLists, RefProblem, TrailerValue, classify_refs};

/// The commit a `scope = "commit"` rule reads — the fields its expressions
/// name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitRecord {
    /// `subject`: the message's first line.
    pub subject: String,
    /// `refs`: the refs the configured trailer names, one per token: a task
    /// stem, a path to a task read as its stem, or an external ref as written.
    pub refs: Vec<String>,
    /// `files`: every changed path, repository-relative.
    pub files: Vec<String>,
    /// The old path of each rename among `files`; `count(files, …)` skips
    /// them, `has` and `matches` still see them.
    pub renamed_from: Vec<String>,
    /// `tasks_only`: the commit changes at least one file, and every one is a
    /// task file (a `.typ` under a `[tasks] roots` entry, outside every
    /// template dir and reserved `.mindtape`/`.git` component).
    pub tasks_only: bool,
    /// `in_range`: the commit was read from a range (`mt check --commits`),
    /// not from a message being written (`mt check --commit-msg`).
    pub in_range: bool,
    /// `is_merge`: the commit has more than one parent. A `commit-msg` check
    /// sets it for the merge being written, an amend of a merge included.
    pub is_merge: bool,
    /// The stems a `scope = "task"` rule runs for: every task file the commit
    /// changes, then every task ref, each once. An external ref is not a task.
    /// Not an expression field.
    pub tasks: Vec<String>,
    /// `trailers`: every trailer line of the message, whatever its key, as
    /// `Key: value`.
    pub trailers: Vec<String>,
}

impl CommitRecord {
    /// Join a read message and its changed files against the loaded `tasks`,
    /// classifying every path through the task-source policy of `config` at
    /// `root`. Pass the state the commit carries, not the working tree: the
    /// policy is lexical, so a staged-only or tracked-deleted file still
    /// classifies as a task.
    ///
    /// Returns the record, plus one [`RefProblem`] per ref that is a path to a
    /// task, or names no task and matches no external pattern.
    #[must_use]
    pub fn build(
        message: &CommitMessage,
        changed: &ChangedFiles,
        tasks: &[Task],
        config: &Config,
        root: &Path,
    ) -> (Self, Vec<RefProblem>) {
        let source = TaskSource::new(root, config);
        let stems: Vec<String> = tasks.iter().map(task_stem).collect();
        let RefLists {
            refs,
            tasks: task_refs,
            problems,
        } = classify_refs(message, &stems, changed, config, &source);
        let tasks_only = !changed.paths.is_empty()
            && changed
                .paths
                .iter()
                .all(|path| is_task_change(path, &changed.root_prefix, &source, &changed.symlinks));
        let mut subjects: Vec<String> = changed
            .paths
            .iter()
            .filter(|path| is_task_change(path, &changed.root_prefix, &source, &changed.symlinks))
            .map(|path| value_stem(path))
            .chain(task_refs)
            .collect();
        dedup_keeping_order(&mut subjects);
        let record = Self {
            subject: message.subject.clone(),
            refs,
            files: changed.paths.clone(),
            renamed_from: changed.renamed_from.clone(),
            tasks_only,
            in_range: false,
            is_merge: false,
            tasks: subjects,
            trailers: message.trailer_lines.clone(),
        };
        (record, problems)
    }
}

/// Drop every repeat of an earlier entry.
fn dedup_keeping_order(values: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::new();
    values.retain(|value| seen.insert(value.clone()));
}

/// A task's stem: its file name without `.typ` — the identity a trailer names.
pub(crate) fn task_stem(task: &Task) -> String {
    task.file.file_stem().map_or_else(
        || task.id.clone(),
        |stem| stem.to_string_lossy().into_owned(),
    )
}

/// One ref's problem, at the line of the trailer that holds it.
pub(super) fn ref_problem_at(token: String, line: usize, found: Problem) -> RefProblem {
    RefProblem {
        value: TrailerValue { value: token, line },
        message: found.text,
        unknown: found.unknown,
    }
}

/// What is wrong with one trailer value, if anything.
pub(super) fn ref_problem(value: &str, stem: &str, stems: &[String], key: &str) -> Option<Problem> {
    let known = stems.iter().any(|known| known == stem);
    if is_path_form(value) {
        let fix = if known {
            format!("write the stem: `{key}: {}`", ref_text(stem))
        } else {
            format!("write a stem, and `{}` names no task", ref_text(stem))
        };
        return Some(format!("`{value}` is a path; {fix}").into());
    }
    (!known).then(|| {
        Problem::new(format!("`{value}` names no task")).with(Unknown::near(
            value,
            &stems.iter().map(String::as_str).collect::<Vec<_>>(),
        ))
    })
}

/// Whether repository-relative `path` is a task file of this project under
/// `source`'s policy. Classification is per path: a path the state holds no
/// regular file for, such as a tracked-deleted path, still counts, while a
/// symlink does not.
pub(crate) fn is_task_change(
    path: &str,
    root_prefix: &str,
    source: &TaskSource,
    symlinks: &[String],
) -> bool {
    is_task_file(path, root_prefix, source) && !symlinks.iter().any(|link| link == path)
}

/// Whether repository-relative `path` is lexically a task file of this
/// project under `source`'s policy. Kind-based classification is
/// [`is_task_change`]'s.
pub(crate) fn is_task_file(path: &str, root_prefix: &str, source: &TaskSource) -> bool {
    project_rel(root_prefix, path)
        .is_some_and(|rel| source.would_be_source(&source.task_path(&rel)))
}
