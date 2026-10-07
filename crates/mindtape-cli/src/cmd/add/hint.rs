//! The `mt add` refusal hint: when the chosen template left a field to a slot
//! and another template writes that same field as the literal the caller's
//! values filled in, and a breach that refused the write is about that field,
//! name that template. A sibling module because `cmd/add/mod.rs` sits at the linecop
//! cap.
//!
//! Nothing here names a field, kind or template: the pins are read off the
//! templates' own `task.with(…)` arguments, so it holds for any project.

use std::collections::BTreeSet;
use std::path::Path;

use mindtape::{Config, discover_slots};

use crate::cmd::add::values::{list_templates, resolve_template};
use crate::task_edit::{Refused, task_with};

/// A field the chosen template left to a slot and the caller passed: its name,
/// the source text it was filled with, and that value unquoted.
type Slotted = (String, String, String);

/// The task fields the breaches that refused the write read
/// ([`mindtape::Violation::reads`]): `Some(None)` when one may be about any
/// field, and `None` when `err` is no such refusal — a source that did not
/// load, or a write refused for its destination, is not fixed by another
/// template's literal.
pub(super) fn refused_fields(err: &anyhow::Error) -> Option<Option<BTreeSet<String>>> {
    err.downcast_ref::<Refused>()
        .map(|refused| refused.reads.clone())
}

/// One `hint:` line per other template pinning a slot-filled field to the
/// value the caller passed for it, or `None` when no template does. `sources`
/// is the chosen template's source (config slots spliced in) before and after
/// the fill; `explicit` holds the slot names the caller passed and `fields`
/// the task fields the refusal is about ([`refused_fields`]).
pub(super) fn template_hint(
    config: &Config,
    root: &Path,
    chosen_name: &str,
    sources: (&str, &str),
    (explicit, fields): (&BTreeSet<String>, Option<&BTreeSet<String>>),
) -> Option<String> {
    let slotted =
        slotted_fields(sources, explicit, fields).filter(|slotted| !slotted.is_empty())?;
    let mut lines = Vec::new();
    for name in list_templates(config, root) {
        if name == chosen_name {
            continue;
        }
        let Ok(source) = resolve_template(config, root, &name) else {
            continue;
        };
        let pins = pins_in(&source, &slotted);
        if !pins.is_empty() {
            lines.push(format!(
                "hint: template `{name}` files {}: mt add -t {name} ...",
                pins.join(", ")
            ));
        }
    }
    (!lines.is_empty()).then(|| lines.join("\n"))
}

/// The fields the refusal is about that `chosen` leaves to a slot the caller
/// passed — the slot may be named differently from its field
/// (`kind: slot("type")`).
fn slotted_fields(
    (chosen, filled): (&str, &str),
    explicit: &BTreeSet<String>,
    fields: Option<&BTreeSet<String>>,
) -> Option<Vec<Slotted>> {
    let (chosen_call, filled_call) = (task_with(chosen).ok()?, task_with(filled).ok()?);
    Some(
        chosen_call
            .args
            .iter()
            .filter_map(|arg| {
                let name = arg.name.as_deref()?;
                if fields.is_some_and(|fields| !fields.contains(name)) {
                    return None;
                }
                let passed = discover_slots(&format!("#{}", &chosen[arg.value_range.clone()]))
                    .into_iter()
                    .any(|slot| explicit.contains(&slot.name));
                if !passed {
                    return None;
                }
                let value = filled_call.named(name)?;
                let text = filled[value.value_range.clone()].to_owned();
                Some((name.to_owned(), text, value.value.clone()))
            })
            .collect(),
    )
}

/// `field=value` for each slotted field `source` writes as that same literal.
fn pins_in(source: &str, slotted: &[Slotted]) -> Vec<String> {
    let Ok(call) = task_with(source) else {
        return Vec::new();
    };
    slotted
        .iter()
        .filter(|(field, text, _)| {
            call.named(field)
                .is_some_and(|arg| source[arg.value_range.clone()] == *text)
        })
        .map(|(field, _, value)| format!("{field}={value}"))
        .collect()
}

/// `--describe` takes its format after `=`, so a word beside it parses as the
/// title and the default template gets described without a word. Refuse it,
/// naming the template when the words spell one.
pub(super) fn refuse_title_beside_describe(
    title: &[String],
    config: &Config,
    root: &Path,
) -> anyhow::Result<()> {
    if title.is_empty() {
        return Ok(());
    }
    let word = title.join(" ");
    if list_templates(config, root).contains(&word) {
        anyhow::bail!("--describe takes no title; did you mean -t {word}?");
    }
    anyhow::bail!("--describe takes no title");
}
