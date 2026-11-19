//! The `mt add` render pipeline after value collection: fill the slots, run
//! `[[on.add]]` hooks over a draft, and apply the non-slot writable fields.
//!
//! The fast path (no hooks and no non-slot field) is today's full fill. Any hook
//! non-slot field needs a draft task the hook's `when` can read. So the
//! pipeline fills the explicit slot values first, stamps status and id once,
//! and leaves the rest of the slots as `slot(...)` calls until the final fill.
//!
//! An interactive add renders inside a loop. A rule that refuses the filled
//! task re-opens the buffer with every breach above the values. The answers
//! survive a rejection the same way they survive a typo.

use std::collections::{BTreeMap, BTreeSet};

use mindtape::model::writable::{self, WritableField};
use mindtape::{AddConfig, RawValue, SlotSpec, discover_slots};

use crate::diag::{Diag, Result};
use crate::pre_write::Problem;
use crate::task_edit::task_with;
use crate::util::Project;

use super::seed::seed;
use super::values::Collected;
use super::{draft, field_values, hooks, inject_header, interactive, markup_safe, target};

/// What one render leaves the caller. It holds the filled source, the values
/// that mint the stem, and the hook-chosen destination. It also holds the
/// explicit field names that the refusal hint needs, and the fields a hook
/// touched.
pub(super) struct Ready {
    pub(super) filled: String,
    pub(super) values: BTreeMap<String, RawValue>,
    pub(super) dir: Option<String>,
    pub(super) explicit: BTreeSet<String>,
    pub(super) env_keys: BTreeSet<String>,
    pub(super) touched: BTreeSet<String>,
    pub(super) interactive: bool,
}

/// What one `mt add` render fills: the template's name and source, the status
/// to stamp, and the check a rule breach re-opens on.
pub(super) struct Request<'a> {
    pub(super) template: &'a str,
    pub(super) source: &'a str,
    pub(super) status: Option<&'a str>,
    pub(super) check: &'a dyn Fn(&Ready) -> Result<Option<Vec<Problem>>>,
}

/// What one interactive round renders: the template, the caller's values and
/// the fields they set. The loop owns the fields, since each round adds the
/// keys the editor changed.
struct Session<'a> {
    args: &'a super::AddArgs,
    project: &'a Project,
    template: &'a str,
    source: &'a str,
    specs: &'a [SlotSpec],
    collected: &'a Collected,
    aliases: &'a BTreeMap<String, String>,
    seed: &'a BTreeMap<String, RawValue>,
    explicit: BTreeSet<String>,
    env_keys: BTreeSet<String>,
    status: Option<&'a str>,
}

impl Session<'_> {
    /// Render the task from `values`, with the fields the caller set so far.
    fn render(&self, values: &BTreeMap<String, RawValue>, interactive: bool) -> Result<Ready> {
        let input = Input {
            args: self.args,
            project: self.project,
            template: self.template,
            source: self.source,
            specs: self.specs,
            collected: self.collected,
            values,
            explicit: &self.explicit,
            status: self.status,
        };
        render_ready(&input, self.env_keys.clone(), interactive)
    }
}

/// Collect the caller's values, run the interactive fill, then [`render`]. An
/// interactive add renders inside the loop, so a rule that refuses the filled
/// task re-opens the buffer instead of parking it.
pub(super) fn collect_and_render(
    args: &super::AddArgs,
    project: &Project,
    request: &Request<'_>,
    tty: bool,
) -> Result<Ready> {
    let config = &project.config;
    let specs = discover_slots(request.source);
    let aliases = field_aliases(request.source);
    let title_slot = title_slot(&aliases);
    let seeded = seed(args, &specs, &aliases, &title_slot)?;
    let interactive =
        interactive::should_open(args.edit, &config.add, &specs, &seeded.values, tty)?;
    let mut session = Session {
        args,
        project,
        template: request.template,
        source: request.source,
        specs: &specs,
        collected: &seeded.collected,
        aliases: &aliases,
        seed: &seeded.values,
        explicit: seeded.explicit,
        env_keys: seeded.env_keys,
        status: request.status,
    };
    if !interactive {
        let mut values = seeded.values.clone();
        ensure_title_value(&mut values, &aliases);
        let ready = session.render(&values, false)?;
        super::notes::report_escaped_spans(&values, &specs);
        return Ok(ready);
    }
    let editor =
        mindtape::allow::runnable_editor(&project.root, config, config.add.editor.as_deref());
    fill_until_it_passes(&mut session, &config.add, editor.as_deref(), request.check)
}

/// The most rounds the editor loop runs before it gives up and lets the final
/// check park the task. A real session fixes a breach in one or two rounds.
/// The bound only stops an editor that keeps changing the buffer without
/// fixing it from spinning forever.
const MAX_ROUNDS: usize = 20;

/// Open the editor until the buffer it saves parses, fills and passes the
/// candidate check. A refusal re-opens the same buffer with every breach above
/// the values. The answers survive a rule the task breaks.
///
/// # Errors
/// Returns an error when the editor fails to spawn or exits without saving, or
/// when the check itself fails.
fn fill_until_it_passes(
    session: &mut Session<'_>,
    config: &AddConfig,
    editor: Option<&str>,
    check: &dyn Fn(&Ready) -> Result<Option<Vec<Problem>>>,
) -> Result<Ready> {
    let fill = interactive::Fill {
        specs: session.specs,
        source: session.source,
        seed: session.seed,
    };
    let mut retry: Option<interactive::Rejected> = None;
    let mut previous: Option<String> = None;
    let mut rounds = 0;
    loop {
        rounds += 1;
        let edited = interactive::fill_until_it_fills(config, editor, &fill, retry.take())?;
        let mut values = edited.values;
        session.explicit.extend(edited.changed);
        ensure_title_value(&mut values, session.aliases);
        let ready = session.render(&values, true)?;
        match check(&ready)? {
            None => {
                super::notes::report_escaped_spans(&values, session.specs);
                return Ok(ready);
            }
            Some(problems) => {
                // An editor that hands back the same buffer cannot fix the
                // breach by re-opening it. Stop, so the final check parks the
                // task and reports the breach instead of spinning. The round
                // bound is the same stop for an editor that keeps changing the
                // buffer without fixing it.
                if previous.as_deref() == Some(edited.buffer.as_str()) || rounds >= MAX_ROUNDS {
                    super::notes::report_escaped_spans(&values, session.specs);
                    return Ok(ready);
                }
                previous = Some(edited.buffer.clone());
                retry = Some(interactive::Rejected::from_problems(
                    edited.buffer,
                    &problems,
                ));
            }
        }
    }
}

/// Render one round and canonicalize its source, so the check and the write
/// see the same bytes.
fn render_ready(input: &Input<'_>, env_keys: BTreeSet<String>, interactive: bool) -> Result<Ready> {
    let rendered = render(input)?;
    Ok(Ready {
        filled: crate::util::canonical_source(&input.project.config, rendered.source),
        values: rendered.values,
        dir: rendered.dir,
        explicit: input.explicit.clone(),
        env_keys,
        touched: rendered.touched,
        interactive,
    })
}

/// The candidate check the interactive loop re-opens on: the problems that
/// refuse the rendered task at the path it would land at, or `None` when it
/// passes. `--no-verify` skips it, as it skips the final check.
pub(super) fn check_candidate(
    args: &super::AddArgs,
    project: &Project,
    template: &str,
    ready: &Ready,
) -> Result<Option<Vec<Problem>>> {
    if args.no_verify {
        return Ok(None);
    }
    let dest = target::resolve_dir(args, project, ready.dir.as_deref())?;
    let stem = target::output_stem(args, &ready.values, &project.config)?;
    let file = dest.join(format!("{stem}.typ"));
    crate::task_edit::candidate_problems(project, &file, &ready.filled, template)
}

/// The template's whole-value slot for a core field, keyed by the slot name:
/// `kind: slot("type")` yields `type`, not `kind`.
pub(super) fn field_aliases(source: &str) -> BTreeMap<String, String> {
    let mut aliases = BTreeMap::new();
    let Ok(call) = task_with(source) else {
        return aliases;
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
        if let [slot] = whole.as_slice() {
            aliases.insert(slot.name.clone(), name.to_owned());
        }
    }
    aliases
}

/// The slot a template uses for the core `title` field, or `title` itself when
/// it has no aliased title slot.
fn title_slot(aliases: &BTreeMap<String, String>) -> String {
    aliases
        .iter()
        .find(|(_, field)| field.as_str() == "title")
        .map_or_else(|| "title".to_owned(), |(slot, _)| slot.clone())
}

/// Keep the core `title` value that mints the stem when the template fills it
/// through an aliased slot. `fill` ignores the extra key: no slot reads it.
fn ensure_title_value(values: &mut BTreeMap<String, RawValue>, aliases: &BTreeMap<String, String>) {
    if values.contains_key("title") {
        return;
    }
    let slot = title_slot(aliases);
    if slot == "title" {
        return;
    }
    if let Some(value) = values.get(&slot).cloned() {
        values.insert("title".to_owned(), value);
    }
}

/// The final source, the slot values the fill used, and the hook-chosen
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
/// A value the template refuses, a draft that does not evaluate, or a hook that
/// cannot run. Also a field write the writer refuses, or an outside-root
/// explicit destination when hooks run.
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
    draft::apply_fields(&mut task.task, &fields)?;
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

/// The explicit field names that a hook may not override. They are the
/// caller's `=` tokens, plus each core field whose whole `task.with(…)` value
/// is a slot the caller set under another name (`kind: slot("type")` with
/// `--set type=bug`). The token names stay as written for the caller-facing
/// hints.
fn hook_explicit(source: &str, explicit: &BTreeSet<String>) -> BTreeSet<String> {
    let mut expanded = explicit.clone();
    for (slot, field) in field_aliases(source) {
        if explicit.contains(&slot) || explicit.contains(&field) {
            expanded.insert(field);
        }
    }
    expanded
}

/// The final source after the hooks. The pipeline upserts every touched field
/// into the still-slot-bearing template, then one fill applies the caller's
/// values, the template's defaults and every value a hook changed.
///
/// Upserting a touched field replaces its whole named argument. A composite
/// expression (`("base",) + slot("tags")`) that a hook appended to loses its
/// embedded slot instead of gaining the hook's value on top. A strict-framework
/// priority a caller set by number loses the leaf slots a hook replaced. The
/// refusal then cites the real problem.
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

#[cfg(test)]
#[path = "pipeline_tests.rs"]
mod tests;
