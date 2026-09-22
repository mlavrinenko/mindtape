//! `mt set <task> <field>=<value>` — write one core field of an existing task.
//!
//! `mt` stamped `status:` (`flip`), `links:` (`link`), `reviewed:` (`review`)
//! and the stem (`mv`), and nothing else. Every other field — a priority worth
//! correcting, a tag worth adding, a due date worth moving — was a hand edit,
//! which is the one path `skills/mindtape/SKILL.md` opens by warning against:
//! it skips what `mt` stamps, resolves and validates, so a wrong value lands
//! and is only reported later, by `mt check`, on a file already saved.
//!
//! So this command is the generic writer for the fields that never earned one.
//! Three things keep it from becoming a second CLI of its own:
//!
//! - the token grammar is `mt add --set`'s, parsed by [`crate::add_tokens`]
//!   itself rather than re-spelled, so `tags+=auth` means here what it means
//!   there and a third operator can never appear on only one of them;
//! - the value becomes a Typst literal through the same [`mindtape::typ_str`]
//!   and [`mindtape::typ_array`] the task emitter uses, so a quote, a
//!   backslash or a one-element array is escaped once, in one place;
//! - the write goes through [`crate::pre_write`], so a value the project
//!   refuses never reaches the file and there is nothing to repair.
//!
//! What it deliberately does not write is [`ELSEWHERE`]: a field with a
//! command of its own, whose argument shape carries more than a value (a
//! dated status, a typed edge, a renamed stem), and `estimate`, whose unit
//! dict has no unambiguous `<field>=<value>` spelling to give it.

use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};
use clap::Args;
use mindtape::{RawValue, Task, format_num, parse_ymd, suggest, typ_array, typ_str};
use typst_edit::apply;

use crate::add_tokens::apply_token;
use crate::pre_write::{self, Pending};
use crate::task_edit::{task_with, upsert_named_arg};
use crate::util;

/// Arguments for `mt set`.
#[derive(Args)]
pub struct SetArgs {
    /// Task to write: slug, relative path, or absolute path.
    pub task: String,
    /// `<field>=<value>`, `<field>+=<value>` to append to a list field, or
    /// `<field>@<path>` to read the value out of a file — `mt add --set`'s
    /// own token grammar. Run `mt set --help` for the writable fields.
    pub assignment: String,
}

/// What a writable field's value looks like once rendered as Typst.
#[derive(Debug, Clone, Copy)]
enum Shape {
    /// A quoted, escaped string literal.
    Text,
    /// A bare number: the raw form of `priority`/`difficulty`. Whether it is
    /// in bounds, or on the configured scale at all, is the validator's call
    /// at [`crate::pre_write`], not this module's.
    Num,
    /// A `YYYY-MM-DD` argument, rendered as the prelude's `on(y, m, d)`.
    Date,
    /// An array literal. The accessor reads the field's current items, which
    /// `+=` appends to; naming it per field is what stops a second list field
    /// from silently appending to `tags`.
    List(fn(&Task) -> &[String]),
}

/// One field `mt set` writes.
#[derive(Debug)]
struct Writable {
    /// The field's name, as the task DSL spells it.
    name: &'static str,
    /// The literal its value renders to.
    shape: Shape,
}

/// Every field `mt set` writes, in the task emitter's own field order: the
/// scalars and lists whose Typst literal is unambiguous. `priority` is the raw
/// numeric form only — a framework priority (`ice(impact: 8, …)`) is a call
/// with its own inputs, and packing those into one `=<value>` token would
/// invent a spelling nothing else in the CLI uses.
const WRITABLE: &[Writable] = &[
    Writable {
        name: "title",
        shape: Shape::Text,
    },
    Writable {
        name: "priority",
        shape: Shape::Num,
    },
    Writable {
        name: "difficulty",
        shape: Shape::Num,
    },
    Writable {
        name: "kind",
        shape: Shape::Text,
    },
    Writable {
        name: "assignee",
        shape: Shape::Text,
    },
    Writable {
        name: "tags",
        shape: Shape::List(|task| &task.tags),
    },
    Writable {
        name: "due",
        shape: Shape::Date,
    },
    Writable {
        name: "start",
        shape: Shape::Date,
    },
];

/// Core fields `mt set` does not write, and what writes them instead. Refused
/// by name rather than as an unknown field: `status` is not a typo for
/// `start`, and a message that suggested so would teach the wrong CLI.
const ELSEWHERE: &[(&str, &str)] = &[
    (
        "status",
        "`mt flip <status> <task>` stamps it dated, against the legend's own transitions",
    ),
    (
        "links",
        "`mt link <task> <kind> <target>` writes typed edges, checking each target",
    ),
    ("reviewed", "`mt review <task>` stamps today's date"),
    (
        "recur",
        "`mt recur <task>` is the only command that advances a recurrence",
    ),
    (
        "estimate",
        "an estimate is a unit dict (`(days: 1, hours: 2)`), not a scalar, so it has no \
         unambiguous `<field>=<value>` spelling",
    ),
    (
        "id",
        "a task's identity is its file stem — `mt mv <task> <new-stem>` renames it and \
         retargets every inbound edge",
    ),
];

/// One parsed assignment: a value to set, or one item to append.
#[derive(Debug)]
enum Assigned {
    /// `<field>=<value>` (or `<field>@<path>`).
    Set(String),
    /// `<field>+=<value>`.
    Append(String),
}

impl Assigned {
    /// The value as the caller typed it — what a refusal or a no-op quotes
    /// back, before any Typst literal was rendered from it.
    fn text(&self) -> &str {
        match self {
            Self::Set(text) | Self::Append(text) => text,
        }
    }
}

/// Run the `set` command.
///
/// # Errors
/// An unresolvable task, a token that is not an assignment, a field this
/// command does not write, a value of the wrong shape for the field, a
/// wrong-shaped file (no `task.with(…)` call), a value the project refuses, or
/// a failed write.
pub fn run(args: &SetArgs) -> Result<()> {
    let project = util::open()?;
    util::warn_errors(&project.outcome);

    let (task, abs) = util::resolve_task(&project, &args.task)?;
    let (name, assigned) = parse_assignment(&args.assignment)?;
    let field = writable(&name)?;
    let Some(value_text) = render(field, &assigned, &task)? else {
        println!(
            "unchanged: {} already carries {:?} in {}",
            task.id,
            assigned.text(),
            field.name
        );
        return Ok(());
    };

    let source =
        std::fs::read_to_string(&abs).with_context(|| format!("read {}", abs.display()))?;
    let call =
        task_with(&source).with_context(|| format!("locate task.with(…) in {}", abs.display()))?;
    let edit = upsert_named_arg(&source, &call, field.name, &value_text)?;
    let updated =
        apply(&source, vec![edit]).with_context(|| format!("apply edit to {}", abs.display()))?;

    // At the file's own path, before the write: a refused value leaves the
    // task exactly as it was — nothing written, nothing to restore.
    pre_write::ensure(
        &project,
        &[Pending {
            file: &abs,
            source: &updated,
        }],
    )?;
    util::write_task(&project.config, &abs, updated)?;
    println!(
        "set {} ({}): {}: {value_text}",
        task.id, task.title, field.name
    );
    Ok(())
}

/// Parse one assignment through `mt add --set`'s own token parser, so the two
/// carriers cannot drift into two grammars.
///
/// [`apply_token`] writes exactly one entry, and from a command line only ever
/// a [`RawValue::Str`] (a `=` or `@` token) or a one-item [`RawValue::List`] (a
/// `+=` one); the richer shapes belong to the `-f` values files `mt set` does
/// not read.
///
/// # Errors
/// A token carrying no operator, an unreadable `@` file, or a value shape no
/// command line produces.
fn parse_assignment(token: &str) -> Result<(String, Assigned)> {
    let mut values = BTreeMap::new();
    apply_token(&mut values, token)?;
    let (name, value) = values
        .into_iter()
        .next()
        .context("no field assignment parsed")?;
    let assigned = match value {
        RawValue::Str(text) => Assigned::Set(text),
        RawValue::List(items) => items
            .into_iter()
            .next_back()
            .map(Assigned::Append)
            .context("no value to append")?,
        _ => bail!("{token:?} is not a <field>=<value> assignment"),
    };
    Ok((name, assigned))
}

/// Every field this command writes, in the emitter's field order.
///
/// Exposed for [`crate::completion`]: the completer for `<field>=<value>` and
/// the refusal below read the same [`WRITABLE`] table, so a tab press can
/// never offer a field this command would then name as unknown.
pub(crate) fn writable_fields() -> impl Iterator<Item = &'static str> {
    WRITABLE.iter().map(|field| field.name)
}

/// The writable field `name` names.
///
/// # Errors
/// A field written by another command (named, with that command), or an
/// unknown one (named, with a nearest-match suggestion and the writable
/// fields listed — the way an unknown `--set` key already is).
fn writable(name: &str) -> Result<&'static Writable> {
    if let Some(field) = WRITABLE.iter().find(|field| field.name == name) {
        return Ok(field);
    }
    if let Some((_, owner)) = ELSEWHERE.iter().find(|(field, _)| *field == name) {
        bail!("`mt set` does not write `{name}`: {owner}");
    }
    let known: Vec<&str> = WRITABLE.iter().map(|field| field.name).collect();
    bail!(
        "unknown field: {name}{}; writable fields: {}",
        suggest::suggestion(name, &known),
        known.join(", ")
    )
}

/// The Typst literal to write, or `None` when the task already says it — a
/// re-run then writes nothing at all, the way a second `mt review` on the same
/// day does.
///
/// # Errors
/// A `+=` on a field that holds no list, or a value the field's shape cannot
/// be made from — refused here, before the file is even read, so the error
/// names the field rather than arriving as a Typst type error later.
fn render(field: &Writable, assigned: &Assigned, task: &Task) -> Result<Option<String>> {
    match (field.shape, assigned) {
        (Shape::List(current), Assigned::Append(item)) => Ok(append(current(task), item)),
        (Shape::List(_), Assigned::Set(text)) => Ok(Some(typ_array(&[typ_str(text)]))),
        (_, Assigned::Append(_)) => bail!(
            "`+=` appends to a list field, and `{name}` is not one — write `{name}=<value>`",
            name = field.name
        ),
        (Shape::Text, Assigned::Set(text)) => Ok(Some(typ_str(text))),
        (Shape::Num, Assigned::Set(text)) => number(field.name, text).map(Some),
        (Shape::Date, Assigned::Set(text)) => date(field.name, text).map(Some),
    }
}

/// `+=`: the field's current items plus `item`, or `None` when it already
/// holds it. Deduping here rather than writing a repeated entry keeps the
/// command re-runnable — a script that appends a tag twice is not a script
/// that meant to tag it twice.
fn append(current: &[String], item: &str) -> Option<String> {
    if current.iter().any(|held| held == item) {
        return None;
    }
    let mut items: Vec<String> = current.iter().map(|held| typ_str(held)).collect();
    items.push(typ_str(item));
    Some(typ_array(&items))
}

/// A bare numeric literal, rendered the way the task emitter renders one.
///
/// # Errors
/// A value that is not a finite number — which no scale could accept either,
/// and is the only numeric mistake this layer can name better than the
/// validator can.
fn number(name: &str, text: &str) -> Result<String> {
    let value: f64 = text
        .parse()
        .ok()
        .filter(|num: &f64| num.is_finite())
        .with_context(|| format!("`{name}` takes a number, got {text:?}"))?;
    Ok(format_num(value))
}

/// A date argument as the prelude's `on(y, m, d)` call.
///
/// # Errors
/// A value that is not a `YYYY-MM-DD` date.
fn date(name: &str, text: &str) -> Result<String> {
    let (year, month, day) = parse_ymd(text)
        .with_context(|| format!("`{name}` takes a YYYY-MM-DD date, got {text:?}"))?;
    Ok(format!("on({year}, {month}, {day})"))
}

#[cfg(test)]
#[path = "set_tests.rs"]
mod tests;
