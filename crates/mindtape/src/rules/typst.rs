//! `[[tasks.rules]] check = { typst, function, field }` calls a project's own
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
//! sits. That is Typst's own import rule, the one [`crate::validate::candidate`]
//! leans on for the opposite reason. A candidate's relative imports must
//! resolve as if the file already lived at its destination. The synthetic file
//! never touches disk: [`typst_world::World::with_main_source`] builds the
//! world around it, the same route [`crate::validate::candidate`] uses for a
//! task that does not exist on disk yet.
//!
//! # Reuse across tasks
//!
//! A project with thousands of tasks and one `check` rule must not re-parse
//! `typst`'s own source (and whatever it imports) once per task. Each task
//! still gets its own [`typst_world::World`], because [`typst::World::main`]
//! takes its value at construction. One `World` cannot serve two different
//! synthetic sources across calls, because its private source cache would
//! serve the first one's text back for the second's id. Every `World` one call
//! builds shares one [`SourceSnapshot`]. That snapshot reads and parses
//! `typst`'s own file (and every file it imports) exactly once, no matter how
//! many tasks ask for it. This is the same pattern [`crate::loader`] uses for
//! its own batch harvest.
//!
//! [`TypstCall`] is that reusable state on its own, so a caller outside the
//! rule engine (an `[[on.add]] derive`) calls the same module through the same
//! loader. [`CompiledCheck`] is one `TypstCall` plus the field it reads.
//! [`validate_call`] is the shared load-time check that the module exists and
//! exports the function.
//!
//! # Value literal
//!
//! The function's argument is [`HVal::to_typst`] of an [`HVal`] tree, which
//! evaluates back to the same value. A [`HVal::Date`] is a real
//! `datetime(year:, month:, day:)` call, not the quoted string
//! [`crate::emit::format::hval_to_typ`] writes into a task file. Only `field`
//! has anything to preserve there, and it reads straight off
//! [`crate::model::Task::extras`] (never round-tripped through JSON, so a
//! nested date the author wrote survives as [`HVal::Date`]). The no-`field`
//! whole-task case comes from the same canonical shape `mt eval --json`
//! prints, whose date-shaped fields are already plain `YYYY-MM-DD` strings by
//! the time [`crate::model::Task`] holds them.
//!
//! # Rules a pack declares
//!
//! A rule a pack declared evaluates in a world rooted at the pack's own
//! directory ([`CheckSpec::root`]). Its file then imports the pack's other
//! files and `@local` packages, never the project's files. The `@local`
//! overrides still come from the project: they name packages, not the
//! project's tree.
//!
//! # Failure
//!
//! The config load refuses a missing file or an unexported function
//! ([`validate_load`]) rather than discovering it per task. Once loaded, every
//! other failure becomes one breach carrying the diagnostic message rather
//! than aborting the project load. That covers a Typst eval error, the function
//! itself calling Typst's `panic()`, and a return value that is not an array of
//! strings. A non-string array element becomes one too, rendered by its own
//! literal.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize as _;
use typst_harvest::{HVal, HarvestWorld, SourceSnapshot, harvest};
use typst_world::root_file_id;

use crate::config::Config;
use crate::model::canonical::CanonicalTask;
use crate::model::{BodyMarker, Task};
use crate::rules::CheckSpec;

/// `path` made canonical, or as given when it can't be.
fn canonical(path: &Path) -> PathBuf {
    crate::canonical(path).unwrap_or_else(|_| path.to_path_buf())
}

/// One `check` rule, built once and reused across every task the rule covers.
/// It holds the project root, the resolved `@local` package overrides, and the
/// [`SourceSnapshot`] that lets every per-task [`typst_world::World`] this rule
/// builds share one parse of `typst`'s own source.
pub(crate) struct CompiledCheck {
    call: TypstCall,
    field: Option<String>,
}

impl CompiledCheck {
    /// Build the reusable state for one `check` rule. It defers all I/O and all
    /// Typst evaluation to the first [`Self::evaluate`] call, so a rule declared
    /// but never reached (a `when` that excludes every task) still pays nothing.
    pub(crate) fn new(spec: &CheckSpec, root: &Path, config: &Config) -> Self {
        let call = TypstCall::new(config, root, &spec.typst, &spec.function);
        let call = match spec.root.as_deref() {
            Some(module_root) => call.rooted_at(module_root),
            None => call,
        };
        Self {
            call,
            field: spec.field.clone(),
        }
    }

    /// Call this rule's function over `task`, returning one string per
    /// breach: empty means the task passes.
    pub(crate) fn evaluate(&self, task: &Task, config: &Config) -> Vec<String> {
        let value = field_value(task, config, self.field.as_deref());
        match self.call.call(&value, "mindtape.check") {
            Ok(value) => breaches_from(&value),
            Err(message) => vec![message],
        }
    }

    /// Call this rule's function over `task` and return its harvested value,
    /// for a collector that publishes it rather than reading breach strings.
    /// The same probe, world and shared snapshot as [`Self::evaluate`].
    pub(crate) fn evaluate_value(&self, task: &Task, config: &Config) -> Result<HVal, String> {
        let value = field_value(task, config, self.field.as_deref());
        self.call.call(&value, "mindtape.value")
    }
}

/// The reusable state behind one project Typst call: the module, the function,
/// the world's root and every resolved `@local` package override, with one
/// [`SourceSnapshot`] shared across calls.
///
/// [`CompiledCheck`] is one of these plus the field it reads, and an
/// `[[on.add]] derive` builds one per call. Both then reach a project's own
/// function through the same loader and the same world.
pub struct TypstCall {
    typst_path: String,
    function: String,
    root_abs: PathBuf,
    overrides: Vec<(String, PathBuf)>,
    snapshot: SourceSnapshot,
    /// Mints a fresh virtual path per call, and each call then gets its own
    /// [`typst::syntax::FileId`]. That matters because a
    /// [`typst_world::World`]'s `main` id takes its value at construction. Its
    /// private source cache would otherwise serve the first call's synthetic
    /// text back for every one after it.
    counter: AtomicU64,
}

impl TypstCall {
    /// Build the reusable state for calling `function` from `typst`. `root` is
    /// the project root: the module resolves from it, and so does every
    /// `@local` package override.
    ///
    /// Defers all I/O and all Typst evaluation to the call itself, so a call
    /// that never happens pays nothing.
    #[must_use]
    pub fn new(config: &Config, root: &Path, typst: &str, function: &str) -> Self {
        Self {
            typst_path: typst.to_owned(),
            function: function.to_owned(),
            root_abs: canonical(root),
            overrides: crate::loader::package_overrides(root, config),
            snapshot: SourceSnapshot::new(),
            counter: AtomicU64::new(0),
        }
    }

    /// Resolve the module from `root` instead of the project root, so a rule a
    /// pack declared reads from the pack's own directory. The `@local`
    /// overrides still come from the project.
    #[must_use]
    pub fn rooted_at(mut self, root: &Path) -> Self {
        self.root_abs = canonical(root);
        self
    }

    /// Call the function on `arg`, returning the value its
    /// `#metadata(…) <label>` marker carries.
    ///
    /// # Errors
    /// A Typst eval error, a function that calls `panic()`, or a call whose
    /// marker never came back.
    pub fn call(&self, arg: &HVal, label: &str) -> Result<HVal, String> {
        let index = self.counter.fetch_add(1, Ordering::Relaxed);
        let id = root_file_id(&format!(".mindtape/check-probe/{index}.typ"))
            .map_err(|err| err.to_string())?;
        let source = format!(
            "#import \"/{}\": {name}\n#metadata({name}({arg})) <{label}>\n",
            self.typst_path,
            name = self.function,
            arg = arg.to_typst(),
        );
        let world = HarvestWorld::with_main_source(id, self.root_abs.clone(), source)
            .with_shared_sources(&self.snapshot);
        let world = crate::loader::with_packages(world, &self.overrides);
        let result = harvest(&world).map_err(|err| err.to_string())?;
        result
            .with_label(label)
            .next()
            .map(|marker| marker.value.clone())
            .ok_or_else(|| "the function produced no value".to_owned())
    }
}

/// Root-aware load-time validation for every `[[tasks.rules]] check` entry:
/// the named Typst module must exist and must export `function`. It complements
/// [`crate::rules::validate_rules`], which validates rule shape without a
/// project root. This one validates the one thing shape-checking cannot see, so
/// it runs from `crate::config::layers::build_config` instead, the one caller
/// that already has root in hand for [`crate::config::compile_dicts`].
///
/// # Errors
/// Returns one joined message, one problem per offending rule, prefixed with
/// its dotted TOML path.
pub(crate) fn validate_collector_load(config: &Config, root: &Path) -> Result<(), String> {
    let mut problems = Vec::new();
    for (index, spec) in config.tasks.collectors.iter().enumerate() {
        if let crate::config::CollectorQuery::Typst(query) = &spec.query
            && let Err(problem) = validate_module(
                root,
                config,
                &query.typst,
                &query.function,
                spec.root.as_deref(),
            )
        {
            problems.push(format!("tasks.collectors[{index}].query: {problem}"));
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}

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
        Err(problems.join("\n"))
    }
}

/// Whether `typst` exists under `root` and exports `function`, for a caller
/// that is not a `[[tasks.rules]] check` entry. An `[[on.add]] derive`
/// validates its call through this, so a hook and a rule share one loader and
/// one set of messages.
///
/// # Errors
/// Returns why the module or function fails to load.
pub fn validate_call(
    config: &Config,
    root: &Path,
    typst: &str,
    function: &str,
) -> Result<(), String> {
    validate_module(root, config, typst, function, None)
}

/// Whether `check.typst` exists and exports `check.function`. A pack's module
/// must sit in the pack's directory, the root its world reads from, so `mt`
/// refuses a path out of it before it looks for the file. Then an explicit
/// filesystem check for the first (a clearer message than whatever Typst's own
/// file-not-found diagnostic says), then a probe import for the second.
/// Neither makes a call, since a function that only errors for some inputs must
/// not fail config load over a value it was never asked to handle.
fn check_module(root: &Path, config: &Config, check: &CheckSpec) -> Result<(), String> {
    validate_module(
        root,
        config,
        &check.typst,
        &check.function,
        check.root.as_deref(),
    )
}

/// Whether `typst` exists under `module_root` (or the project root) and
/// exports `function`.
fn validate_module(
    root: &Path,
    config: &Config,
    typst: &str,
    function: &str,
    module_root: Option<&Path>,
) -> Result<(), String> {
    let check_root = module_root.unwrap_or(root);
    let module = check_root.join(typst);
    if module_root.is_some() && !crate::config::pack_dirs::stays_inside(check_root, &module) {
        return Err(format!("`{typst}` is outside the pack's directory"));
    }
    if !module.is_file() {
        return Err(format!("`{typst}` does not exist"));
    }
    let root_abs = canonical(check_root);
    let id = root_file_id(".mindtape/check-probe/load.typ").map_err(|err| err.to_string())?;
    let source = format!("#import \"/{typst}\": {function}\n");
    let world = HarvestWorld::with_main_source(id, root_abs, source);
    let world =
        crate::loader::with_packages(world, &crate::loader::package_overrides(root, config));
    harvest(&world)
        .map(|_| ())
        .map_err(|err| format!("`{typst}` does not export `{function}`: {err}"))
}

/// The value `check.function` receives for `task`. `markers` is the one
/// reserved input, read straight off [`Task::markers`] before anything else: no
/// canonical field and no `x-<ns>` slot carries it. Otherwise the value is the
/// raw extras value when `field` names an `x-<ns>` slot (see the module doc's
/// "Value literal" section for why that keeps a nested [`HVal::Date`] intact).
/// Without such a slot it is a lookup into the same canonical shape
/// `mt eval --json` prints, or, with no `field`, the whole canonical task.
fn field_value(task: &Task, config: &Config, field: Option<&str>) -> HVal {
    let Some(name) = field else {
        return canonical_hval(task, config);
    };
    if name == super::MARKERS {
        return markers_hval(&task.markers);
    }
    if let Some(value) = task.extras.get(name) {
        return value.clone();
    }
    canonical_hval(task, config)
        .get(name)
        .cloned()
        .unwrap_or(HVal::Null)
}

/// The reserved `markers` check input: one `(label, value)` dict per body
/// marker, in document order, with `label` as `none` for an unlabelled marker.
/// A task that emits none yields an empty array, never `none`. A check can
/// then loop over it without a nil guard.
fn markers_hval(markers: &[BodyMarker]) -> HVal {
    HVal::Array(
        markers
            .iter()
            .map(|marker| {
                let label = marker.label.clone().map_or(HVal::Null, HVal::Str);
                HVal::Dict(BTreeMap::from([
                    ("label".to_owned(), label),
                    ("value".to_owned(), marker.value.clone()),
                ]))
            })
            .collect(),
    )
}

/// `task` projected to the same canonical shape `mt eval --json` prints, as an
/// [`HVal`] tree instead of [`serde_json::Value`]. It routes through
/// [`CanonicalTask`]'s JSON rather than a second projection off [`Task`], so
/// the two never drift on what "the whole task" contains.
fn canonical_hval(task: &Task, config: &Config) -> HVal {
    let canonical = CanonicalTask::from_task(task, config);
    serde_json::to_value(canonical)
        .and_then(HVal::deserialize)
        .unwrap_or(HVal::Null)
}

/// Fields an `[[on.add]]` hook may not read off the draft: the identity, the
/// destination, and every value a load computes from either.
const ADD_UNSAFE: &[&str] = &[
    "id",
    "file",
    "path",
    "dir",
    "ready",
    "blocked",
    "depth",
    "topo",
    "blocks",
    "children",
    "children_done",
    "progress",
    "derived_tags",
    "derived",
];

/// Whether `name` is an [`ADD_UNSAFE`] field. An `[[on.add]]` `when` reads the
/// same synthetic draft a derive does, so it must refuse these too.
#[must_use]
pub(crate) fn add_unsafe(name: &str) -> bool {
    ADD_UNSAFE.contains(&name)
}

/// The value an `[[on.add]] derive` function receives: the draft's canonical
/// task minus [`ADD_UNSAFE`], plus `template`.
///
/// The clean shape is the one `mt eval --json` prints, routed through
/// [`CanonicalTask`], so a hook's function and a rule's `check` read the same
/// field names and shapes. It cannot see the draft's synthetic path, because
/// the destination is exactly what the hook is about to choose.
#[must_use]
pub fn add_argument(task: &Task, config: &Config, template: &str) -> HVal {
    let mut draft = serde_json::to_value(CanonicalTask::from_task(task, config))
        .ok()
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    for key in ADD_UNSAFE {
        draft.remove(*key);
    }
    draft.insert(
        "template".to_owned(),
        serde_json::Value::String(template.to_owned()),
    );
    HVal::deserialize(serde_json::Value::Object(draft)).unwrap_or(HVal::Null)
}

/// Read the harvested `mindtape.check` value back into a breach list. An array
/// projects element by element: a string element verbatim, any other shape by
/// its own literal repr. That is total rather than erroring on a function that
/// returned the wrong element type. Anything else is one breach that reports
/// what came back.
fn breaches_from(value: &HVal) -> Vec<String> {
    match value {
        HVal::Array(items) => items.iter().map(string_or_literal).collect(),
        other => vec![format!("returned {}", other.to_typst())],
    }
}

fn string_or_literal(item: &HVal) -> String {
    item.as_str()
        .map_or_else(|| item.to_typst(), ToOwned::to_owned)
}

#[cfg(test)]
#[path = "typst_tests.rs"]
mod tests;
