//! The `mt add` refusal hint: when the chosen template left a field to a slot
//! and another template writes that same field as the literal the caller's
//! values filled in, name that template. A sibling module because `add.rs`
//! sits at the linecop cap.
//!
//! Nothing here names a field, kind or template: the pins are read off the
//! templates' own `task.with(…)` arguments, so it holds for any project.

use std::collections::BTreeSet;
use std::path::Path;

use mindtape::Config;

use crate::add_values::{list_templates, resolve_template};
use crate::task_edit::task_with;

/// A field the chosen template left to a slot and the caller passed: its name,
/// the source text it was filled with, and that value unquoted.
type Slotted = (String, String, String);

/// One `hint:` line per other template pinning a slot-filled field to the
/// value the caller passed for it, or `None` when no template does. `sources`
/// is the chosen template's source (config slots spliced in) before and after
/// the fill; `explicit` holds the keys the caller passed.
pub(super) fn template_hint(
    config: &Config,
    root: &Path,
    chosen_name: &str,
    sources: (&str, &str),
    explicit: &BTreeSet<String>,
) -> Option<String> {
    let slotted = slotted_fields(sources, explicit)?;
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

/// The explicitly passed fields that `chosen` leaves to a slot.
fn slotted_fields(
    (chosen, filled): (&str, &str),
    explicit: &BTreeSet<String>,
) -> Option<Vec<Slotted>> {
    let (chosen_call, filled_call) = (task_with(chosen).ok()?, task_with(filled).ok()?);
    Some(
        chosen_call
            .args
            .iter()
            .filter_map(|arg| {
                let name = arg.name.as_deref()?;
                if !explicit.contains(name) || !chosen[arg.value_range.clone()].contains("slot(") {
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
