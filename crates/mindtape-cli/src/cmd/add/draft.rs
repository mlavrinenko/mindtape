//! The partially filled task `mt add` runs its `[[on.add]]` hooks over.
//!
//! A hook's `when` reads a whole task, but `mt add` has only a template and the
//! values the caller supplied. So `mt add` stamps the add-owned status and id,
//! fills the explicit slot values, and evaluates the result in memory at
//! `<first root>/__draft__.typ`. It never writes that file. Every slot the
//! caller left unfilled keeps its `slot(...)` call, which evaluates to its
//! `default`. The draft then loads without a fully filled task.
//!
//! The path still feeds Typst's relative imports and the `depth`/`topo` a load
//! computes from the folder. `mt add` clears only the folder-derived tags after
//! the harvest, because a hook's `when` reads the task it is about to file, not
//! the synthetic draft path. `depth` and `topo` stay, and config load refuses an
//! add `when` that reads a field the draft does not carry.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use mindtape::model::writable;
use mindtape::{Candidate, RawValue, SlotSpec, Task};

use crate::diag::{Diag, Hint, Kind, Result};
use crate::util::Project;

use super::inject_header;

/// A draft task and the source that produced it.
pub(super) struct Draft {
    /// The task the draft evaluates as, with derived tags cleared.
    pub(super) task: Task,
    /// The header-injected template with every slot still in place. The final
    /// fill then sees a `slot(...)` call for each value a hook may have
    /// changed.
    pub(super) source: String,
}

/// Fill the caller's explicit slot values, stamp status and id once, and
/// evaluate the result as a draft task.
///
/// # Errors
/// A first root with no project-relative path, a source that does not evaluate,
/// a value the template refuses, or a header edit that fails.
pub(super) fn build(
    project: &Project,
    source: &str,
    values: &BTreeMap<String, RawValue>,
    specs: &[SlotSpec],
    status: Option<&str>,
) -> Result<Draft> {
    let source = inject_header(source, &project.config, status)?;
    let partial =
        mindtape::fill_partial(&source, &super::markup_safe(values, specs)).map_err(Diag::from)?;
    let rel = draft_rel(project)?;
    let candidate = Candidate {
        rel: &rel,
        source: &partial,
    };
    let mut task = mindtape::harvest_candidate(&project.root, &project.config, candidate)
        .map_err(|err| draft_error(&err))?;
    task.derived_tags.clear();
    Ok(Draft { task, source })
}

/// Apply the caller's non-slot writable fields to a draft.
///
/// # Errors
/// A value the writer refuses for its field.
pub(super) fn apply_fields(
    task: &mut Task,
    fields: &[(&'static writable::WritableField, Vec<writable::FieldOp>)],
) -> Result<()> {
    for (field, ops) in fields {
        for op in ops {
            writable::apply_op(task, field, op).map_err(Diag::from)?;
        }
    }
    Ok(())
}

/// The synthetic draft path, relative to the project root.
fn draft_rel(project: &Project) -> Result<PathBuf> {
    let first = project
        .config
        .tasks
        .roots
        .first()
        .map_or(".", mindtape::RootSpec::path);
    let root = Path::new(first);
    let relative = match root.strip_prefix(&project.root) {
        Ok(rel) => rel.to_path_buf(),
        Err(_) if root.is_absolute() => return Err(outside_root(first)),
        Err(_) => root.to_path_buf(),
    };
    let escapes = relative
        .components()
        .any(|part| !matches!(part, Component::Normal(_) | Component::CurDir));
    if escapes {
        return Err(outside_root(first));
    }
    Ok(relative.join("__draft__.typ"))
}

/// A clear refusal for a first root that gives the draft no project-relative
/// path. Naming the config key rather than the raw path is what tells a reader
/// where the fix belongs.
fn outside_root(root: &str) -> Diag {
    Diag::failed("the first tasks root must lie under the project root")
        .value("tasks.roots")
        .hint(Hint::naming(
            "the draft is evaluated under the first root, and this one is",
            mindtape::display_path(Path::new(root)),
        ))
}

/// A draft evaluation failure, placed and worded like any other load error.
fn draft_error(err: &mindtape::LoadError) -> Diag {
    let diag = Diag::verbatim(Kind::Failed, err.message.clone());
    crate::warnings::at_place(diag, &err.file, err.line, err.column)
        .typst_hints(err.hints.iter().cloned())
}

#[cfg(test)]
#[path = "draft_tests.rs"]
mod tests;
