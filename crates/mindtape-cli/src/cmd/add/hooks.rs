//! `[[on.add]]` execution: the ordered actions that fill a new task and choose
//! where it lands.
//!
//! One mutable draft task flows through the hook list in declaration order, so
//! each `when` sees what the earlier hooks left. `set` and `append` go through
//! the shared writable writer ([`mindtape::model::writable`]); `derive` calls a
//! project Typst function over the sanitized draft ([`mindtape::rules::typst`])
//! and writes its value back; `dir` is expanded after them in the same action,
//! over the draft's scalar values, and the last hook that names one wins.
//!
//! An explicit `=` from the caller owns its field: a hook `set` or `derive` on
//! it is skipped and reported, while an explicit `+=` does not block a later
//! `set`. Hook appends always accumulate, and a later hook's `set` replaces the
//! draft value (and so drops earlier appends), because the draft is the one
//! source of truth both the next `when` and the final fill read.

use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

use mindtape::config::add_hooks::{
    AddAction, AddHook, AddValue, DeriveAction, DirTemplate, DirValue, desugar_add_when,
    validate_derived_dir,
};
use mindtape::model::writable::{self, FieldOp, Shape, WritableField};
use mindtape::rules::typst::{self, TypstCall};
use mindtape::{Config, Filter, RawValue, Readiness, Task};
use typst_harvest::HVal;

use crate::diag::{Diag, Kind, Result, emit_text};

/// What the hook list chose: the last `dir`, and every field an action wrote.
#[derive(Debug, Default)]
pub(super) struct Outcome {
    /// The last hook's expanded `dir`, project-root-relative.
    pub(super) dir: Option<String>,
    /// Every field a hook set, appended or derived.
    pub(super) touched: BTreeSet<String>,
}

/// The caller's own input, which a hook may not override: the fields an
/// explicit `=` owns, and whether `--dir` already chose the destination.
pub(super) struct Input<'a> {
    /// The project root a derive's Typst file is resolved from.
    pub(super) root: &'a Path,
    /// The template `mt add` used, readable as `{template}` and `template ==`.
    pub(super) template: &'a str,
    /// The fields an explicit `=` from the caller owns.
    pub(super) explicit_sets: &'a BTreeSet<String>,
    /// An explicit `--dir` wins, so a hook's `dir` is not expanded at all.
    pub(super) explicit_dir: bool,
}

/// Run every `[[on.add]]` hook whose `when` matches `draft`, in order.
///
/// # Errors
/// A `when` that cannot be parsed or evaluated, a value the shared writer
/// refuses, a derive call that fails or returns the wrong type, or a `dir`
/// whose macro has no value or expands to a path component the template cannot
/// hold.
pub(super) fn run(config: &Config, input: &Input<'_>, draft: &mut Task) -> Result<Outcome> {
    let mut outcome = Outcome::default();
    let pre_hook = draft.clone();
    for (index, hook) in config.on.add.iter().enumerate() {
        let label = label(index, hook);
        let run = Run {
            root: input.root,
            template: input.template,
            explicit_set: input.explicit_sets,
            explicit_dir: input.explicit_dir,
            config,
            label: &label,
        };
        if !fires(hook, draft, &pre_hook, &run)? {
            continue;
        }
        for action in &hook.actions {
            apply(action, &run, draft, &mut outcome)?;
        }
    }
    Ok(outcome)
}

/// The label a report gives a hook: its name, or its position.
fn label(index: usize, hook: &AddHook) -> String {
    hook.name
        .clone()
        .unwrap_or_else(|| format!("on.add[{index}]"))
}

/// Whether this hook's `when` matches the current draft.
fn fires(hook: &AddHook, draft: &Task, old: &Task, run: &Run<'_>) -> Result<bool> {
    let when = desugar_add_when(&hook.when, run.template)
        .map_err(|message| error(run.label, format!("can't read the when: {message}")))?;
    let filter = Filter::parse(&when).map_err(Diag::from)?;
    filter
        .matches_add(draft, old, run.config, &Readiness::empty())
        .map_err(Diag::from)
}

/// What every action of one hook run needs, grouped so `apply` stays under the
/// argument limit.
struct Run<'a> {
    root: &'a Path,
    template: &'a str,
    explicit_set: &'a BTreeSet<String>,
    explicit_dir: bool,
    config: &'a Config,
    label: &'a str,
}

/// Apply one action: its sets, appends, derive, then its `dir`.
fn apply(action: &AddAction, run: &Run<'_>, draft: &mut Task, outcome: &mut Outcome) -> Result<()> {
    for (name, value) in &action.set {
        set(name, value, run, draft, outcome)?;
    }
    for (name, items) in &action.append {
        append(name, items, run, draft, outcome)?;
    }
    if let Some(derive) = &action.derive {
        field_derive(derive, run, draft, outcome)?;
    }
    if let Some(dir) = &action.dir
        && !run.explicit_dir
    {
        apply_dir(dir, run, draft, outcome)?;
    }
    Ok(())
}

/// Apply a `set`, unless the caller's explicit `=` owns the field.
fn set(
    name: &str,
    value: &AddValue,
    run: &Run<'_>,
    draft: &mut Task,
    outcome: &mut Outcome,
) -> Result<()> {
    let field = field(name, run.label)?;
    if run.explicit_set.contains(name) {
        emit_text(&format!(
            "on.add {}: {name} keeps the explicit value\n",
            run.label
        ));
        return Ok(());
    }
    writable::apply_op(draft, field, &FieldOp::Set(value.raw()), run.config)
        .map_err(|err| field_error(err, name, run.label))?;
    outcome.touched.insert(name.to_owned());
    emit_text(&format!("on.add {}: set {name}\n", run.label));
    Ok(())
}

/// Apply an `append`: one item at a time, deduped by the writer.
fn append(
    name: &str,
    items: &[String],
    run: &Run<'_>,
    draft: &mut Task,
    outcome: &mut Outcome,
) -> Result<()> {
    let field = field(name, run.label)?;
    for item in items {
        writable::apply_op(draft, field, &FieldOp::Append(item.clone()), run.config)
            .map_err(|err| field_error(err, name, run.label))?;
    }
    outcome.touched.insert(name.to_owned());
    emit_text(&format!("on.add {}: append {name}\n", run.label));
    Ok(())
}

/// Apply a `derive`: call the function over the sanitized draft and replace the
/// field with what it returns. An explicit `=` still wins.
fn field_derive(
    derive: &DeriveAction,
    run: &Run<'_>,
    draft: &mut Task,
    outcome: &mut Outcome,
) -> Result<()> {
    let Some(name) = derive.field.as_deref() else {
        return Ok(());
    };
    let field = field(name, run.label)?;
    if run.explicit_set.contains(name) {
        emit_text(&format!(
            "on.add {}: {name} keeps the explicit value\n",
            run.label
        ));
        return Ok(());
    }
    let call = Derivation::of(&derive.typst, &derive.function);
    let value = call.evaluate(run, draft)?;
    let raw = field_value(field, &value).ok_or_else(|| call.type_error(name, run.label))?;
    writable::apply_op(draft, field, &FieldOp::Set(raw), run.config)
        .map_err(|err| call.field_error(err, name, run.label))?;
    outcome.touched.insert(name.to_owned());
    emit_text(&format!("on.add {}: derive {name} via {call}\n", run.label));
    Ok(())
}

/// Apply a `dir`: expand a static template, or call a derive function and
/// validate the relative path it returns.
fn apply_dir(dir: &DirValue, run: &Run<'_>, draft: &mut Task, outcome: &mut Outcome) -> Result<()> {
    match dir {
        DirValue::Path(text) => {
            let expanded = expand(text, run.template, draft, run.label)?;
            emit_text(&format!("on.add {}: dir = {expanded}\n", run.label));
            outcome.dir = Some(expanded);
        }
        DirValue::Derived { derive } => {
            let call = Derivation::of(&derive.typst, &derive.function);
            let value = call.evaluate(run, draft)?;
            let dir = value
                .as_str()
                .ok_or_else(|| call.type_error("dir", run.label))?;
            validate_derived_dir(dir).map_err(|message| call.fail(&message, run.label))?;
            emit_text(&format!("on.add {}: dir via {call}\n", run.label));
            outcome.dir = Some(dir.to_owned());
        }
    }
    Ok(())
}

/// The writable field an action names, or a clear refusal.
fn field(name: &str, label: &str) -> Result<&'static WritableField> {
    writable::get(name)
        .ok_or_else(|| error(label, format!("an action names an unknown field: {name}")))
}

/// One hook's failure, prefixed with the hook's name or position.
fn error(label: &str, message: impl fmt::Display) -> Diag {
    Diag::verbatim(Kind::Failed, format!("on.add {label}: {message}"))
}

/// An action's write refusal, placed on the hook that asked for it.
fn field_error(err: writable::WritableError, name: &str, label: &str) -> Diag {
    let diag = Diag::from(err).rule(format!("on.add {label}"));
    diag.value(format!("{name}: {label}"))
}

/// A derived value as the writer's input shape. A list field takes a Typst
/// array of strings and nothing else; a scalar field takes one scalar. `None`
/// is a value the field cannot hold.
fn field_value(field: &WritableField, value: &HVal) -> Option<RawValue> {
    if matches!(field.shape, Shape::List(_)) {
        let items: Option<Vec<String>> = value
            .as_array()?
            .iter()
            .map(|item| item.as_str().map(ToOwned::to_owned))
            .collect();
        return items.map(RawValue::List);
    }
    match value {
        HVal::Str(text) | HVal::Date(text) => Some(RawValue::Str(text.clone())),
        HVal::Int(number) => Some(RawValue::Int(*number)),
        HVal::Float(number) => Some(RawValue::Float(*number)),
        HVal::Bool(flag) => Some(RawValue::Bool(*flag)),
        HVal::Null | HVal::Array(_) | HVal::Dict(_) => None,
    }
}

/// Expand a `dir` template over the draft's current scalar values.
fn expand(text: &str, template: &str, draft: &Task, label: &str) -> Result<String> {
    let parsed = DirTemplate::parse(text)
        .map_err(|message| error(label, format!("can't read the dir: {message}")))?;
    parsed
        .expand(|name| macro_value(name, template, draft))
        .map_err(|message| error(label, format!("can't choose the task directory: {message}")))
}

/// One `dir` macro's value: the template name, or a writable scalar's text.
fn macro_value(name: &str, template: &str, draft: &Task) -> Option<String> {
    if name == "template" {
        return Some(template.to_owned());
    }
    let field = writable::get(name)?;
    match writable::current_raw(field, draft)? {
        RawValue::Str(text) => Some(text),
        _ => None,
    }
}

/// One derive call's module and function: the pair every report and the
/// [`TypstCall`] itself need.
#[derive(Debug, Clone, Copy)]
struct Derivation<'a> {
    typst: &'a str,
    function: &'a str,
}

impl<'a> Derivation<'a> {
    /// The call `typst` and `function` name.
    fn of(typst: &'a str, function: &'a str) -> Self {
        Self { typst, function }
    }

    /// Call the function over the sanitized draft.
    fn evaluate(&self, run: &Run<'_>, draft: &Task) -> Result<HVal> {
        let call = TypstCall::new(run.config, run.root, self.typst, self.function);
        let argument = typst::add_argument(draft, run.config, run.template);
        call.call(&argument, "mindtape.derive")
            .map_err(|message| self.fail(&message, run.label))
    }

    /// A value the field cannot hold, naming the hook, field and function.
    fn type_error(&self, name: &str, label: &str) -> Diag {
        Diag::failed("a derive returned a value the field cannot hold")
            .value(format!("{name} via {self}"))
            .rule(format!("on.add {label}"))
    }

    /// A write the shared writer refused, named like [`Self::type_error`].
    fn field_error(&self, err: writable::WritableError, name: &str, label: &str) -> Diag {
        Diag::from(err)
            .value(format!("{name} via {self}"))
            .rule(format!("on.add {label}"))
    }

    /// A failed call in the hook's words, keeping Typst's message.
    fn fail(&self, message: &str, label: &str) -> Diag {
        error(label, format!("{self}: {message}"))
    }
}

impl fmt::Display for Derivation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.typst, self.function)
    }
}

#[cfg(test)]
#[path = "hooks_tests.rs"]
mod tests;
