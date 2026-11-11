//! `[[on.add]]` — defaults a project or a pack applies to a newly added task.
//!
//! ```toml
//! [[on.add]]
//! name = "fixes"
//! when = 'template == "fix"'
//! actions = [{ set = { due = "2026-12-01" }, dir = "tasks/fixes" }]
//! ```
//!
//! `when` is a [filter expression](crate::filter) over the task being added.
//! `template == "name"` and `template != "name"` compare the template `mt add`
//! used, and `from_template("name")` is the same test as `==`. A bare
//! `template`, another operator or a non-literal name is a config error, as is
//! a name no configured template dir holds. An empty `when` fires on every add.
//! Like `into(...)` on a flip, the two forms are rewritten into a
//! filter-language constant before parsing ([`desugar_add_when`]), so the
//! evaluator stays unaware of them.
//!
//! Each action sets at least one of `set`, `append`, `derive` and `dir`. The
//! first two name writable fields ([`crate::model::writable`]) and are
//! validated through the same writer `mt set` uses; `dir` is not a field, so
//! `set.dir` is refused by name. `dir` is a project-root-relative template over
//! `{template}` and the add-safe scalar fields, expanded by the add pipeline
//! with [`DirTemplate::expand`]; an unknown macro or an absolute path is a
//! config error, and the empty string is the project root.
//!
//! `derive = { typst, function, field }` calls a project's own Typst function
//! and replaces `field` with what it returns, the same call a rule's `check`
//! makes ([`crate::rules::typst`]). It needs `field`, which must be writable and
//! absent from the same action's `set` and `append`. `dir = { derive = { typst,
//! function } }` derives the destination instead; it takes no `field`, and the
//! function returns a project-root-relative path. Both forms are checked for a
//! module that exists and exports the function at config load, through the same
//! loader a `check` uses ([`crate::rules::typst::validate_call`]).
//!
//! `[[on.add]]` merges by name across packs and layers like `[[on.flip]]`
//! ([`crate::config::packs`]): a nearer entry with the same name replaces an
//! earlier one in place, `disable` drops it, and an unnamed entry appends.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Deserialize;

use crate::config::Config;
pub use crate::config::add_hooks_when::{desugar_add_when, when_templates};
use crate::model::Task;
use crate::model::writable::{self, FieldOp, Shape};
use crate::template::RawValue;

/// The built-in template every project has, even with no `templates.dirs`.
pub const BUILTIN_TEMPLATE: &str = "task";

/// One `[[on.add]]` entry: when it fires, and what it does.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct AddHook {
    /// Optional name. A later entry with the same name, from a nearer layer or
    /// a pack, replaces an earlier one in place, and `disable` drops it by
    /// name. An unnamed entry always appends. Also the label
    /// `mt config show --origin` prints.
    pub name: Option<String>,
    /// Filter over the task being added, plus the two template tests. Empty
    /// fires on every add.
    #[schemars(extend("x-expression" = "filter"))]
    pub when: String,
    /// Actions to apply, in declaration order.
    pub actions: Vec<AddAction>,
}

/// One action of an `[[on.add]]` entry. Set at least one of `set`, `append`,
/// `derive` and `dir`.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct AddAction {
    /// Fields to set, by writable name. `dir` is not a field and is refused.
    pub set: BTreeMap<String, AddValue>,
    /// List fields to append to, by writable name.
    pub append: BTreeMap<String, Vec<String>>,
    /// Where the task's file goes, as a project-root-relative template or a
    /// derived path.
    pub dir: Option<DirValue>,
    /// A field to replace with a value a Typst function returns.
    pub derive: Option<DeriveAction>,
}

/// `derive = { typst, function, field }`: call `function` from the `typst`
/// module and replace `field` with its value. A list field takes the whole
/// returned array, which must hold only strings.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeriveAction {
    /// Path to the module exporting `function`, relative to the project root.
    pub typst: String,
    /// Name of the function the module exports.
    pub function: String,
    /// The writable field the returned value replaces.
    #[serde(default)]
    pub field: Option<String>,
}

/// A value a hook sets: the scalar shapes a TOML config writes naturally.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(untagged)]
pub enum AddValue {
    /// A string, including a date.
    Text(String),
    /// An integer.
    Int(i64),
    /// A float.
    Float(f64),
    /// A boolean, rendered as `true` or `false`.
    Bool(bool),
    /// An array of strings.
    List(Vec<String>),
}

impl AddValue {
    /// This value as the writer's own input shape.
    #[must_use]
    pub fn raw(&self) -> RawValue {
        match self {
            Self::Text(text) => RawValue::Str(text.clone()),
            Self::Int(number) => RawValue::Int(*number),
            Self::Float(number) => RawValue::Float(*number),
            Self::Bool(flag) => RawValue::Bool(*flag),
            Self::List(items) => RawValue::List(items.clone()),
        }
    }
}

/// A `dir` action's value: a static template, or a path a Typst function
/// derives.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(untagged)]
pub enum DirValue {
    /// A project-root-relative template.
    Path(String),
    /// A destination computed by a Typst function.
    Derived {
        /// The function to call.
        derive: DerivedDir,
    },
}

/// `dir = { derive = { typst, function } }`: call `function` from the `typst`
/// module and use the relative path it returns. It takes no `field`; the
/// destination is the value.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DerivedDir {
    /// Path to the module exporting `function`, relative to the project root.
    pub typst: String,
    /// Name of the function the module exports.
    pub function: String,
}

impl DirValue {
    /// The template text, when this is the static form.
    #[must_use]
    pub fn template(&self) -> Option<&str> {
        match self {
            Self::Path(template) => Some(template),
            Self::Derived { .. } => None,
        }
    }

    /// The derive call, when this is the derived form.
    #[must_use]
    pub fn derived(&self) -> Option<&DerivedDir> {
        match self {
            Self::Derived { derive } => Some(derive),
            Self::Path(_) => None,
        }
    }
}

/// Every template `mt add` accepts: the built-in `task` plus every `*.typ`
/// stem under `config.templates.dirs`. The CLI's `mt add --list` reads this
/// same list, so the two cannot drift.
#[must_use]
pub fn template_names(config: &Config, root: &Path) -> Vec<String> {
    let mut names = BTreeSet::new();
    names.insert(BUILTIN_TEMPLATE.to_owned());
    for dir in config.templates.normalized_dirs() {
        let Ok(entries) = std::fs::read_dir(root.join(dir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("typ")
                && let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
            {
                names.insert(stem.to_owned());
            }
        }
    }
    names.into_iter().collect()
}

use crate::config::add_hooks_dir::validate_dir;
pub use crate::config::add_hooks_dir::{DirTemplate, validate_component, validate_derived_dir};

/// Whether `name` is a writable scalar a `dir` template may expand: `text`,
/// number and date fields, except the ones that come from a file's location.
#[must_use]
pub fn is_add_safe(name: &str) -> bool {
    writable::get(name).is_some_and(|field| {
        matches!(field.shape, Shape::Text | Shape::Num | Shape::Date)
            && !crate::filter::path_dependent_fields().contains(&name)
    })
}

/// Check every `[[on.add]]` entry at config load, returning one message per
/// problem, prefixed with its dotted TOML path.
///
/// `root` is the project root the template names, `dir` paths and derive
/// modules resolve against.
///
/// # Errors
/// Returns the joined problems when a hook names an unknown template, field or
/// macro, a value the shared writer refuses, a field in both `set` and
/// `append` (or in the action's `derive`), `set.dir`, an absolute `dir`, or a
/// derive whose module or function does not load.
pub fn validate_add_hooks(config: &Config, root: &Path) -> Result<(), String> {
    let names = template_names(config, root);
    let known: Vec<&str> = names.iter().map(String::as_str).collect();
    let mut checker = Checker {
        known: &known,
        probe: Task::default(),
        config,
        root,
        problems: Vec::new(),
    };
    for (index, hook) in config.on.add.iter().enumerate() {
        checker.hook(hook, &format!("on.add[{index}]"));
    }
    if checker.problems.is_empty() {
        Ok(())
    } else {
        Err(checker.problems.join("\n"))
    }
}

/// The state one config load's validation accumulates.
struct Checker<'a> {
    known: &'a [&'a str],
    probe: Task,
    config: &'a Config,
    root: &'a Path,
    problems: Vec<String>,
}

impl Checker<'_> {
    /// Validate one hook's `when` and actions.
    fn hook(&mut self, hook: &AddHook, at: &str) {
        self.when(hook, at);
        if hook.actions.is_empty() {
            self.problems.push(format!("{at}: no actions declared"));
        }
        for (slot, action) in hook.actions.iter().enumerate() {
            self.action(action, &format!("{at}.actions[{slot}]"));
        }
    }

    /// Validate the `when`: its template names, then the filter itself.
    fn when(&mut self, hook: &AddHook, at: &str) {
        let when_at = format!("{at}.when");
        let (desugared, names) = match crate::config::add_hooks_when::rewrite(&hook.when) {
            Ok(pair) => pair,
            Err(message) => {
                self.problems.push(format!("{when_at}: {message}"));
                return;
            }
        };
        for name in names {
            if !self.known.contains(&name.as_str()) {
                self.problems.push(format!(
                    "{when_at}: unknown template: {}",
                    crate::config::quote(&name)
                ));
            }
        }
        // Read the source, not the rewrite: a template test desugars to
        // `has(id)`/`miss(id)`, which would read as unsafe.
        self.add_unsafe(&hook.when, &when_at);
        if let Some(problem) = crate::filter::expr_problem_add(&desugared, &self.probe, self.config)
        {
            self.problems.push(problem.under(&when_at).to_string());
        }
    }

    /// Refuse a `when` that reads a field the synthetic draft cannot answer:
    /// its identity, destination and every value a load computes from them.
    fn add_unsafe(&mut self, source: &str, when_at: &str) {
        let mut roots = BTreeSet::new();
        for name in crate::filter::names_in(source, self.config) {
            if let Some(root) = name.split(['.', '[']).next()
                && crate::rules::typst::add_unsafe(root)
            {
                roots.insert(root.to_owned());
            }
        }
        for name in crate::filter::calls_in(source, self.config) {
            if crate::filter::is_set_function(&name) {
                roots.insert(name);
            }
        }
        for root in roots {
            self.problems.push(format!(
                "{when_at}: an add when cannot read `{root}`: it is not known until the task \
                 is filed"
            ));
        }
    }

    /// Validate one action's `set`, `append`, `derive` and `dir`.
    fn action(&mut self, action: &AddAction, at: &str) {
        if action.set.is_empty()
            && action.append.is_empty()
            && action.derive.is_none()
            && action.dir.is_none()
        {
            self.problems
                .push(format!("{at}: set, append, derive or dir is required"));
            return;
        }
        for (name, value) in &action.set {
            self.set(name, value, at);
        }
        for (name, items) in &action.append {
            self.append(name, items, at);
        }
        for name in action.append.keys() {
            if action.set.contains_key(name) {
                self.problems
                    .push(format!("{at}: {name} is in both set and append"));
            }
        }
        if let Some(derive) = &action.derive {
            self.derive(derive, action, at);
        }
        if let Some(dir) = &action.dir {
            self.dir(dir, at);
        }
    }

    /// Validate one `derive`: a writable field the action does not also `set`
    /// or `append`, and a module and function that load.
    fn derive(&mut self, derive: &DeriveAction, action: &AddAction, at: &str) {
        let Some(name) = derive.field.as_deref() else {
            self.problems
                .push(format!("{at}.derive.field: a derive action needs a field"));
            return;
        };
        if writable::get(name).is_none() {
            self.problems
                .push(format!("{at}.derive.field: unknown writable field: {name}"));
        }
        if action.set.contains_key(name) {
            self.problems
                .push(format!("{at}: {name} is in both derive and set"));
        }
        if action.append.contains_key(name) {
            self.problems
                .push(format!("{at}: {name} is in both derive and append"));
        }
        if let Err(message) = self.call(&derive.typst, &derive.function) {
            self.problems.push(format!("{at}.derive: {message}"));
        }
    }

    /// Validate one `dir` action: a static template, or a derive call.
    fn dir(&mut self, dir: &DirValue, at: &str) {
        match dir {
            DirValue::Path(text) => {
                if let Err(message) = validate_dir(text) {
                    self.problems.push(format!("{at}.dir: {message}"));
                }
            }
            DirValue::Derived { derive } => {
                if let Err(message) = self.call(&derive.typst, &derive.function) {
                    self.problems.push(format!("{at}.dir.derive: {message}"));
                }
            }
        }
    }

    /// Whether a derive's module exists and exports its function, through the
    /// loader a rule's `check` uses.
    fn call(&self, typst: &str, function: &str) -> Result<(), String> {
        crate::rules::typst::validate_call(self.config, self.root, typst, function)
    }

    /// Validate one `set` entry through the shared writer.
    fn set(&mut self, name: &str, value: &AddValue, at: &str) {
        if name == "dir" {
            self.problems
                .push(format!("{at}.set.dir: dir is a destination, not a field"));
            return;
        }
        let Some(field) = writable::get(name) else {
            self.problems
                .push(format!("{at}.set.{name}: unknown writable field"));
            return;
        };
        if let Err(error) = writable::render_op(field, &FieldOp::Set(value.raw()), &self.probe) {
            self.problems.push(format!("{at}.set.{name}: {error}"));
        }
    }

    /// Validate one `append` entry: a writable list field, each item through
    /// the shared writer.
    fn append(&mut self, name: &str, items: &[String], at: &str) {
        let Some(field) = writable::get(name) else {
            self.problems
                .push(format!("{at}.append.{name}: unknown writable field"));
            return;
        };
        if !matches!(field.shape, Shape::List(_)) {
            self.problems
                .push(format!("{at}.append.{name}: += appends to a list field"));
            return;
        }
        // A list field's append has no failure mode the shape check above did
        // not already name; the call still goes through the shared writer so
        // the validator and the runtime read one renderer.
        for item in items {
            let _ = writable::render_op(field, &FieldOp::Append(item.clone()), &self.probe);
        }
    }
}

#[cfg(test)]
#[path = "add_hooks_tests.rs"]
mod tests;
