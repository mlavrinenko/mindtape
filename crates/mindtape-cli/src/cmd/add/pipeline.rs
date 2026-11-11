//! The `mt add` render pipeline after values are collected: fill the slots,
//! run `[[on.add]]` hooks over a draft, and apply the fields no slot holds.
//!
//! The fast path — no hooks and no non-slot field — is today's full fill. Any
//! hook or non-slot field needs a draft task the hook's `when` can read, so the
//! explicit slot values are filled first, status and id are stamped once, and
//! the rest of the slots stay as `slot(...)` calls until the final fill.

use std::collections::{BTreeMap, BTreeSet};
use std::io::IsTerminal;

use mindtape::model::writable::{self, FieldOp, WritableField};
use mindtape::{RawValue, SlotSpec, discover_slots};

use crate::diag::{Diag, Hint, Result};
use crate::task_edit::task_with;
use crate::util::Project;

use super::title::{fill_title_from_args, fill_title_from_stdin, split_positional};
use super::values::{Collected, collect_values, layer_env, reject_unknown_keys};
use super::{draft, field_values, hooks, inject_header, interactive, markup_safe, target};

/// What one render leaves the caller: the filled source, the values the stem
/// is minted from, the hook-chosen destination, the explicit field names the
/// refusal hint needs, and the fields a hook touched.
pub(super) struct Ready {
    pub(super) filled: String,
    pub(super) values: BTreeMap<String, RawValue>,
    pub(super) dir: Option<String>,
    pub(super) explicit: BTreeSet<String>,
    pub(super) env_keys: BTreeSet<String>,
    pub(super) touched: BTreeSet<String>,
    pub(super) interactive: bool,
}

/// Collect the caller's values, run the interactive fill, then [`render`].
pub(super) fn collect_and_render(
    args: &super::AddArgs,
    project: &Project,
    template: &str,
    source: &str,
    status: Option<&str>,
) -> Result<Ready> {
    let config = &project.config;
    let specs = discover_slots(source);
    let (title_words, tags) = split_positional(args, &specs);
    let mut collected = collect_values(&args.values, &args.set, &args.tokens)?;
    let tag_ops = tags
        .into_iter()
        .map(|tag| ("tags".to_owned(), FieldOp::Append(tag)));
    collected.ops.splice(0..0, tag_ops);
    let flags = flag_ops(args);
    refuse_flag_and_set(args, &flags)?;
    collected.ops.extend(flags);
    let mut explicit = super::explicit_sets(&collected.ops);
    reject_unknown_keys(&collected.ops, &specs)?;
    let mut values = collected.slot_values(&specs)?;
    if fill_title_from_args(&mut values, &title_words) {
        explicit.insert("title".to_owned());
    }
    let before_env: BTreeSet<String> = values.keys().cloned().collect();
    layer_env(&mut values, &specs, |key| std::env::var(key).ok());
    let env_keys: BTreeSet<String> = values
        .keys()
        .filter(|key| !before_env.contains(*key))
        .cloned()
        .collect();
    if fill_title_from_stdin(&mut values, args)? {
        explicit.insert("title".to_owned());
    }
    let tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let interactive = interactive::should_open(args.edit, &config.add, &specs, &values, tty)?;
    if interactive {
        let editor =
            mindtape::allow::runnable_editor(&project.root, config, config.add.editor.as_deref());
        let edited = interactive::run(&config.add, editor.as_deref(), &specs, &values)?;
        explicit.extend(edited.changed);
        values = edited.values;
    }
    let input = Input {
        args,
        project,
        template,
        source,
        specs: &specs,
        collected: &collected,
        values: &values,
        explicit: &explicit,
        status,
    };
    let rendered = render(&input)?;
    super::notes::report_escaped_spans(&values, &specs);
    Ok(Ready {
        filled: rendered.source,
        values: rendered.values,
        dir: rendered.dir,
        explicit,
        env_keys,
        touched: rendered.touched,
        interactive,
    })
}

/// Every scalar `mt add` flag as an ordered field write, in the same shape
/// `--set field=value` builds.
fn flag_ops(args: &super::AddArgs) -> Vec<(String, FieldOp)> {
    [
        ("priority", &args.priority),
        ("difficulty", &args.difficulty),
        ("kind", &args.kind),
        ("assignee", &args.assignee),
        ("due", &args.due),
        ("start", &args.start),
    ]
    .into_iter()
    .filter_map(|(name, value)| {
        value
            .as_ref()
            .map(|value| (name.to_owned(), FieldOp::Set(RawValue::Str(value.clone()))))
    })
    .collect()
}

/// Refuse a scalar flag and a `--set`/after-`--` token naming the same field:
/// two spellings of one value is a caller mistake, not a precedence rule.
fn refuse_flag_and_set(args: &super::AddArgs, flags: &[(String, FieldOp)]) -> Result<()> {
    for (name, _) in flags {
        let repeated = args
            .set
            .iter()
            .chain(&args.tokens)
            .any(|token| super::tokens::token_name(token).is_ok_and(|token| token == name));
        if repeated {
            return Err(Diag::usage("the same field is set twice")
                .value(name.clone())
                .hint(Hint::new("pass the value once, as a flag or as --set")));
        }
    }
    Ok(())
}

/// The final source, the slot values it was filled from, and the hook-chosen
/// destination.
pub(super) struct Rendered {
    /// The rendered task source.
    pub(super) source: String,
    /// The final slot values, including a value a hook set, for the stem.
    pub(super) values: BTreeMap<String, RawValue>,
    /// The last hook's `dir`, project-root-relative.
    pub(super) dir: Option<String>,
    /// Every field a hook filled, so a note can tell a hook's value from
    /// config's. Empty on the fast path.
    pub(super) touched: BTreeSet<String>,
}

/// Everything one render reads, grouped to keep the call under the argument
/// limit.
pub(super) struct Input<'a> {
    pub(super) args: &'a super::AddArgs,
    pub(super) project: &'a Project,
    pub(super) template: &'a str,
    pub(super) source: &'a str,
    pub(super) specs: &'a [SlotSpec],
    pub(super) collected: &'a Collected,
    pub(super) values: &'a BTreeMap<String, RawValue>,
    pub(super) explicit: &'a BTreeSet<String>,
    pub(super) status: Option<&'a str>,
}

/// Fill, hook and field-write one new task.
///
/// # Errors
/// A value the template refuses, a draft that does not evaluate, a hook that
/// cannot run, a field write the writer refuses, or an outside-root explicit
/// destination when hooks are enabled.
pub(super) fn render(input: &Input<'_>) -> Result<Rendered> {
    let Input {
        args,
        project,
        template,
        source,
        specs,
        collected,
        values,
        explicit,
        status,
    } = *input;
    let fields = field_values::touched(collected, specs);
    let hooks_on = !args.no_hooks && !project.config.on.add.is_empty();
    if !hooks_on && fields.is_empty() {
        let filled = mindtape::fill(source, &markup_safe(values, specs)).map_err(Diag::from)?;
        let filled = inject_header(&filled, &project.config, status)?;
        return Ok(Rendered {
            source: filled,
            values: values.clone(),
            dir: None,
            touched: BTreeSet::new(),
        });
    }

    if hooks_on {
        target::refuse_outside_explicit(args.dir.as_deref(), project)?;
    }
    let mut task = draft::build(project, source, values, specs, status)?;
    draft::apply_fields(&mut task.task, &fields, project)?;
    let outcome = if hooks_on {
        let hooked_explicit = hook_explicit(source, explicit);
        let hook_input = hooks::Input {
            root: &project.root,
            template,
            explicit_sets: &hooked_explicit,
            explicit_dir: args.dir.is_some(),
        };
        hooks::run(&project.config, &hook_input, &mut task.task)?
    } else {
        hooks::Outcome::default()
    };

    let touched = field_names(&fields, &outcome.touched);
    finish(&task, specs, values, &touched, outcome.dir)
}

/// The explicit field names a hook may not override: the caller's `=` tokens,
/// plus each core field whose whole `task.with(…)` value is a slot the caller
/// set under another name (`kind: slot("type")` with `--set type=bug`). The
/// token names stay as written for the caller-facing hints.
fn hook_explicit(source: &str, explicit: &BTreeSet<String>) -> BTreeSet<String> {
    let mut expanded = explicit.clone();
    let Ok(call) = task_with(source) else {
        return expanded;
    };
    for arg in &call.args {
        let Some(name) = arg.name.as_deref() else {
            continue;
        };
        if writable::get(name).is_none() {
            continue;
        }
        let value = source.get(arg.value_range.clone()).unwrap_or_default();
        let hashed = format!("#{value}");
        let whole: Vec<SlotSpec> = discover_slots(&hashed)
            .into_iter()
            .filter(|slot| slot.range.start == 1 && slot.range.end == hashed.len())
            .collect();
        if let [slot] = whole.as_slice()
            && explicit.contains(&slot.name)
        {
            expanded.insert(name.to_owned());
        }
    }
    expanded
}

/// The final source after the hooks: every touched field is upserted into the
/// still-slot-bearing template, then one fill applies the caller's values, the
/// template's defaults and every value a hook changed.
///
/// Upserting a touched field replaces its whole named argument. A composite
/// expression (`("base",) + slot("tags")`) a hook appended to therefore loses
/// its embedded slot instead of being filled on top of the hook's value, and a
/// strict-framework priority a caller set by number loses the leaf slots a
/// hook replaced, so the refusal names the real problem.
fn finish(
    task: &draft::Draft,
    specs: &[SlotSpec],
    values: &BTreeMap<String, RawValue>,
    touched: &BTreeSet<String>,
    dir: Option<String>,
) -> Result<Rendered> {
    let mut final_values = values.clone();
    for name in touched {
        if let Some(field) = writable::get(name)
            && specs.iter().any(|spec| spec.name == *name)
            && let Some(raw) = writable::current_raw(field, &task.task)
        {
            final_values.insert(name.clone(), raw);
        }
    }
    let upsert: Vec<&'static WritableField> = touched
        .iter()
        .filter_map(|name| writable::get(name))
        .collect();
    let source = field_values::upsert(&task.source, &upsert, &task.task)?;
    let filled = mindtape::fill(&source, &markup_safe(&final_values, specs)).map_err(Diag::from)?;
    if !final_values.contains_key("title") && !task.task.title.trim().is_empty() {
        final_values.insert("title".to_owned(), RawValue::Str(task.task.title.clone()));
    }
    Ok(Rendered {
        source: filled,
        values: final_values,
        dir,
        touched: touched.clone(),
    })
}

/// Every field the caller's non-slot operations and the hooks touched.
fn field_names(
    fields: &[(&'static WritableField, Vec<writable::FieldOp>)],
    hook_touched: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = field_values::names(fields)
        .into_iter()
        .map(str::to_owned)
        .collect();
    names.extend(
        hook_touched
            .iter()
            .filter_map(|name| writable::get(name).map(|field| field.name.to_owned())),
    );
    names
}
