//! The context threaded through the filter walk and the field resolver.
//!
//! Split out of `filter/mod.rs` for the file-size gate. [`Ctx`] carries the
//! task under test, the config, the readiness oracle, the `in_view` depth, the
//! dry-run flag, and — for an `[[on.add]]` `when` only — the pre-hook draft
//! `old("field")` reads.

use crate::config::Config;
use crate::filter::{FilterError, Readiness};
use crate::model::Task;
use crate::model::fields::{self, FieldValue};

/// Shared context threaded through the filter walk: the task under test, the
/// config (framework scores, terminal set, view lookup), the readiness oracle
/// (`ready` / `blocked`), the current `in_view` recursion depth, whether this
/// is the load-time dry run, and the pre-hook draft when one exists.
pub(crate) struct Ctx<'a> {
    pub(crate) task: &'a Task,
    pub(crate) config: &'a Config,
    pub(crate) readiness: &'a Readiness,
    pub(crate) depth: usize,
    pub(crate) dry: bool,
    /// Whether the task exists in the commit state being read — `Some` only
    /// under a task-scope commit rule's `head(...)` / `staged(...)` /
    /// `worktree(...)`, where `exists` means something.
    pub(crate) exists: Option<bool>,
    /// The task before any `[[on.add]]` action ran. `Some` only inside an
    /// add hook's `when`; every other caller leaves it `None`, so `old()`
    /// refuses there instead of inventing a value.
    pub(crate) old: Option<&'a Task>,
}

impl<'a> Ctx<'a> {
    /// A context reading `task` as one commit state holds it
    /// ([`crate::commit::task_expr`]): `exists` answers whether the state holds
    /// the file at all, `readiness` is computed over that state's task set.
    pub(crate) fn in_state(
        task: &'a Task,
        config: &'a Config,
        readiness: &'a Readiness,
        exists: bool,
        dry: bool,
    ) -> Self {
        Ctx {
            task,
            config,
            readiness,
            depth: 0,
            dry,
            exists: Some(exists),
            old: None,
        }
    }

    /// The same context evaluating `task` instead — what `count` hands its
    /// argument for each task it counts.
    pub(crate) fn over<'b>(&self, task: &'b Task) -> Ctx<'b>
    where
        'a: 'b,
    {
        Ctx {
            task,
            config: self.config,
            readiness: self.readiness,
            depth: self.depth,
            dry: self.dry,
            // Every task of a state's set is present in it.
            exists: self.exists.map(|_| true),
            // The pre-hook draft belongs to the task being added, not to a
            // sibling `count(...)` walks.
            old: None,
        }
    }

    /// The same context one `in_view` level deeper.
    pub(crate) fn deeper(&self) -> Ctx<'a> {
        Ctx {
            task: self.task,
            config: self.config,
            readiness: self.readiness,
            depth: self.depth + 1,
            dry: self.dry,
            exists: self.exists,
            old: self.old,
        }
    }
}

/// Resolve a field name to its value on `task`, via the shared
/// [`crate::model::fields`] registry — the single place every generically
/// comparable field is wired, so a field usable here is usable in `--sort` too.
/// `pub(crate)` so [`crate::filter::predicates`]'s `present`/`contains_value`
/// can share it.
pub(crate) fn field_value(
    task: &Task,
    name: &str,
    config: &Config,
) -> Result<FieldValue, FilterError> {
    if let Some(value) = crate::filter::predicates::estimate_unit(task, name, config) {
        return value;
    }
    if fields::is_derived(name) {
        return fields::derived(task, name, config)
            .ok_or_else(|| FilterError::UnknownField(name.to_owned()));
    }
    if fields::is_extra(name) {
        return crate::filter::predicates::extra_value(task, name);
    }
    fields::get(name)
        .map(|def| (def.get)(task, config))
        .ok_or_else(|| FilterError::UnknownField(name.to_owned()))
}
