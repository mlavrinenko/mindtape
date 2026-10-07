//! `[[tasks.rules]] check = { typst, function, field }` — call a project's own
//! Typst function over one task, turning its returned array of strings into
//! breaches.
//!
//! # Evaluation route
//!
//! For each task asked, a synthetic source imports `function` from `typst`
//! and wraps its result in `#metadata(...) <mindtape.check>`:
//!
//! ```text
//! #import "/truth/lib.typ": check-goals
//! #metadata(check-goals(("product": (…),))) <mindtape.check>
//! ```
//!
//! The import path is absolute (`/`-rooted), so it resolves against the
//! project root regardless of where the synthetic file's own virtual path
//! sits — Typst's own import rule, the one [`crate::validate::candidate`] leans on for
//! the opposite reason (a candidate's relative imports must resolve as if the
//! file already lived at its destination). The synthetic file never touches
//! disk: it is installed with
//! [`typst_world::World::with_source_override`], the same route
//! [`crate::validate::candidate`] uses for a task that does not exist on disk yet, on a
//! world built with [`typst_world::DiskProvider`] rooted at the project.
//!
//! # Reuse across tasks
//!
//! A project with thousands of tasks and one `check` rule must not re-parse
//! `typst`'s own source (and whatever it imports) once per task. Each task
//! still gets its own [`typst_world::World`] — [`typst::World::main`] is fixed
//! at construction, so one `World` cannot serve two different synthetic
//! sources across calls without its private source cache serving the first
//! one's text back for the second's id — but every `World` [`CompiledCheck`]
//! builds for one rule shares one [`SourceSnapshot`], so `typst`'s own file
//! (and every file it imports) is read and parsed exactly once no matter how
//! many tasks ask for it. This is the same pattern [`crate::loader`] uses for
//! its own batch harvest.
//!
//! # Value literal
//!
//! The function's argument is a Typst literal built from an [`HVal`] tree,
//! reusing [`crate::emit::format`]'s string/array/dict rendering with one
//! difference: [`HVal::Date`] renders as a real `datetime(year:, month:,
//! day:)` call rather than the quoted string [`crate::emit::format::hval_to_typ`]
//! writes for round-tripping `x-<ns>` extras verbatim. That distinction only
//! has anything to preserve for `field`, which reads straight off
//! [`crate::model::Task::extras`] (never round-tripped through JSON, so a
//! nested date the author wrote survives as [`HVal::Date`]); the no-`field`
//! whole-task case is built from the same canonical shape `mt eval --json`
//! prints, whose date-shaped fields are already plain `YYYY-MM-DD` strings by
//! the time [`crate::model::Task`] holds them.
//!
//! # Failure
//!
//! A missing file or an unexported function is refused at config load
//! ([`validate_load`]), not discovered per task. Once loaded, every other
//! failure — a Typst eval error, or the function itself calling Typst's
//! `panic()` — becomes one breach carrying the diagnostic message rather than
//! aborting the project load; so does a return value that is not an array of
//! strings, and a non-string array element repr's itself into one.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;
use typst_harvest::{HVal, Harvest, HarvestWorld, SourceSnapshot, harvest};
use typst_world::{DiskProvider, root_file_id};

use crate::config::Config;
use crate::emit::format::{hval_to_typ, typ_array, typ_dict_with};
use crate::model::Task;
use crate::model::canonical::{CanonicalTask, json_to_hval};
use crate::rules::CheckSpec;

/// One `check` rule, built once and reused across every task it is asked
/// about: the project root, the resolved `@local` package overrides, and the
/// [`SourceSnapshot`] that lets every per-task [`typst_world::World`] this
/// rule builds share one parse of `typst`'s own source.
pub(crate) struct CompiledCheck {
    typst_path: String,
    function: String,
    field: Option<String>,
    root_abs: PathBuf,
    overrides: Vec<(String, PathBuf)>,
    snapshot: SourceSnapshot,
    /// Mints a fresh virtual path per task, so each gets its own [`typst::syntax::FileId`]
    /// — required because a [`typst_world::World`]'s `main` id is fixed at
    /// construction, and its private source cache would otherwise serve the
    /// first task's synthetic text back for every one after it.
    counter: AtomicU64,
}

impl CompiledCheck {
    /// Build the reusable state for one `check` rule. Does no IO and no Typst
    /// evaluation itself — the first cost lands on the first
    /// [`Self::evaluate`] call, so a rule declared but never reached (a `when`
    /// that excludes every task) still pays nothing.
    pub(crate) fn new(spec: &CheckSpec, root: &Path, config: &Config) -> Self {
        Self {
            typst_path: spec.typst.clone(),
            function: spec.function.clone(),
            field: spec.field.clone(),
            root_abs: root.canonicalize().unwrap_or_else(|_| root.to_path_buf()),
            overrides: crate::loader::package_overrides(root, config),
            snapshot: SourceSnapshot::new(),
            counter: AtomicU64::new(0),
        }
    }

    /// Call this rule's function over `task`, returning one string per
    /// breach: empty means the task passes.
    pub(crate) fn evaluate(&self, task: &Task, config: &Config) -> Vec<String> {
        let value = field_value(task, config, self.field.as_deref());
        let Some((id, source)) = self.probe(&value) else {
            return vec!["internal error: could not build a check probe".to_owned()];
        };
        let mut world =
            HarvestWorld::with_provider(id, self.root_abs.clone(), Box::new(DiskProvider))
                .with_package_root(crate::loader::package_root())
                .with_shared_sources(&self.snapshot);
        for (name, dir) in &self.overrides {
            world = world.with_local_package(name, dir.clone());
        }
        let world = world.with_source_override(id, source);
        match harvest(&world) {
            Ok(result) => breaches_from(&result),
            Err(err) => vec![err.to_string()],
        }
    }

    /// A fresh, project-rooted virtual path and the synthetic source it backs:
    /// `#import` `function` from `typst_path` and call it on `value`, wrapped
    /// in a labelled marker [`breaches_from`] reads back out.
    fn probe(&self, value: &HVal) -> Option<(typst_syntax::FileId, String)> {
        let index = self.counter.fetch_add(1, Ordering::Relaxed);
        let id = root_file_id(&format!(".mindtape/check-probe/{index}.typ")).ok()?;
        let source = format!(
            "#import \"/{}\": {name}\n#metadata({name}({arg})) <mindtape.check>\n",
            self.typst_path,
            name = self.function,
            arg = check_literal(value),
        );
        Some((id, source))
    }
}

/// Root-aware load-time validation for every `[[tasks.rules]] check` entry:
/// the named Typst module must exist and must export `function`. Complements
/// [`crate::rules::validate_rules`], which validates rule shape
/// without a project root — this validates the one thing shape-checking
/// cannot see, so it runs from [`crate::config::layers::build_config`] instead,
/// the one caller that already has root in hand for [`crate::config::compile_dicts`].
///
/// # Errors
/// Returns one joined message, one problem per offending rule, prefixed with
/// its dotted TOML path.
pub(crate) fn validate_load(config: &Config, root: &Path) -> Result<(), String> {
    let mut problems = Vec::new();
    for (index, spec) in config.tasks.rules.iter().enumerate() {
        if let Some(check) = &spec.check
            && let Err(problem) = check_module(root, config, check)
        {
            problems.push(format!("tasks.rules[{index}].check: {problem}"));
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

/// Whether `check.typst` exists and exports `check.function`: an explicit
/// filesystem check for the first (a clearer message than whatever Typst's
/// own file-not-found diagnostic says), then a probe import for the second —
/// no call, since a function that only errors for some inputs must not fail
/// config load over a value it was never asked to handle.
fn check_module(root: &Path, config: &Config, check: &CheckSpec) -> Result<(), String> {
    if !root.join(&check.typst).is_file() {
        return Err(format!("`{}` does not exist", check.typst));
    }
    let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let id = root_file_id(".mindtape/check-probe/load.typ").map_err(|err| err.to_string())?;
    let mut world = HarvestWorld::with_provider(id, root_abs, Box::new(DiskProvider))
        .with_package_root(crate::loader::package_root());
    for (name, dir) in crate::loader::package_overrides(root, config) {
        world = world.with_local_package(&name, dir);
    }
    let source = format!("#import \"/{}\": {}\n", check.typst, check.function);
    let world = world.with_source_override(id, source);
    harvest(&world).map(|_| ()).map_err(|err| {
        format!(
            "`{}` does not export `{}`: {err}",
            check.typst, check.function
        )
    })
}

/// The value `check.function` receives for `task`: the raw extras value when
/// `field` names an `x-<ns>` slot (see the module doc's "Value literal"
/// section for why that keeps a nested [`HVal::Date`] intact), else a lookup
/// into the same canonical shape `mt eval --json` prints, else — no `field` —
/// the whole canonical task.
fn field_value(task: &Task, config: &Config, field: Option<&str>) -> HVal {
    let Some(name) = field else {
        return canonical_hval(task, config);
    };
    if let Some(value) = task.extras.get(name) {
        return value.clone();
    }
    canonical_hval(task, config)
        .get(name)
        .cloned()
        .unwrap_or(HVal::Null)
}

/// `task` projected to the same canonical shape `mt eval --json` prints, as an
/// [`HVal`] tree instead of [`serde_json::Value`] — routed through
/// [`CanonicalTask`] and [`json_to_hval`] rather than a second projection off
/// [`Task`], so the two never drift on what "the whole task" contains.
fn canonical_hval(task: &Task, config: &Config) -> HVal {
    let canonical = CanonicalTask::from_task(task, config);
    let json = serde_json::to_value(canonical).unwrap_or(Value::Null);
    json_to_hval(&json)
}

/// Read the harvested `mindtape.check` marker back into a breach list: an
/// array projects element by element (a string element verbatim, any other
/// shape by its own literal repr — total rather than erroring on a function
/// that returned the wrong element type); anything else is one breach naming
/// what came back.
fn breaches_from(result: &Harvest) -> Vec<String> {
    let Some(marker) = result.with_label("mindtape.check").next() else {
        return vec!["the check produced no result".to_owned()];
    };
    match &marker.value {
        HVal::Array(items) => items.iter().map(string_or_literal).collect(),
        other => vec![format!("returned {}", check_literal(other))],
    }
}

fn string_or_literal(item: &HVal) -> String {
    item.as_str()
        .map_or_else(|| check_literal(item), ToOwned::to_owned)
}

/// Project one [`HVal`] to the Typst literal `check.function` receives: like
/// [`hval_to_typ`], except a date renders as a real `datetime(...)` value
/// instead of a quoted string (see the module doc's "Value literal" section).
fn check_literal(value: &HVal) -> String {
    match value {
        HVal::Date(text) => date_literal(text),
        HVal::Array(items) => typ_array(&items.iter().map(check_literal).collect::<Vec<_>>()),
        HVal::Dict(map) => typ_dict_with(map, check_literal),
        other => hval_to_typ(other),
    }
}

/// `datetime(year: Y, month: M, day: D)` for a well-formed `YYYY-MM-DD`
/// [`HVal::Date`]; falls back to a quoted string for one that is not (an
/// [`HVal::Date`] is only ever built from a real Typst `datetime`, by
/// [`typst_world::convert`], so this is defensive rather than reachable in
/// practice).
fn date_literal(text: &str) -> String {
    crate::date::parse_ymd(text).map_or_else(
        || hval_to_typ(&HVal::Str(text.to_owned())),
        |(year, month, day)| format!("datetime(year: {year}, month: {month}, day: {day})"),
    )
}

#[cfg(test)]
#[path = "typst_tests.rs"]
mod tests;
