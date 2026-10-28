//! The `mt add` refusal hint: when the chosen template left a field to a slot
//! and another template writes that same field as the literal the caller's
//! values filled in, and a breach that refused the write is about that field,
//! name that template. A sibling module because `cmd/add/mod.rs` sits at the linecop
//! cap.
//!
//! Nothing here names a field, kind or template: the pins are read off the
//! templates' own `task.with(…)` arguments, so it holds for any project.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use mindtape::{Config, discover_slots};

use crate::cmd::add::values::{list_templates, resolve_template};
use crate::diag::{Command, Diag, Hint, Note};
use crate::task_edit::task_with;

/// A field the chosen template left to a slot and the caller passed: its name,
/// the source text it was filled with, and that value unquoted.
type Slotted = (String, String, String);

/// What a rejected task is reported and kept with: the template it was filled
/// from, which names it in a refusal, whether the values came from the
/// `--edit` buffer, and where a rejected task was parked.
pub(super) struct Rescue {
    pub template: String,
    pub interactive: bool,
    pub parked: std::cell::Cell<Option<std::path::PathBuf>>,
}

impl Rescue {
    pub(super) fn new(template: &str, interactive: bool) -> Self {
        Self {
            template: template.to_owned(),
            interactive,
            parked: std::cell::Cell::new(None),
        }
    }
}

/// What the refusal of a filled task is judged against: the chosen template,
/// its source before and after the fill, and the slot names the caller passed.
pub(super) struct RefusalContext<'a> {
    pub config: &'a Config,
    pub root: &'a Path,
    pub template: &'a str,
    /// The chosen template's source (config slots spliced in) before and after
    /// the fill.
    pub sources: (&'a str, &'a str),
    pub explicit: &'a BTreeSet<String>,
}

impl RefusalContext<'_> {
    /// `err` as a [`Diag`] with what the caller can act on: a note naming where
    /// the rejected `--edit` buffer was `parked`, and, when the refusal is
    /// about a field another template files as the value the caller passed, a
    /// hint to use that template.
    pub(super) fn decorate(&self, err: Diag, parked: Option<PathBuf>) -> Diag {
        // A refusal carries the fields its breaches read, for the template hint.
        let refused = err.reads().cloned();
        let mut diag = err;
        if let Some(parked) = parked {
            diag = diag.note(
                Note::new("the filled task was parked at").value(mindtape::display_path(&parked)),
            );
        }
        if let Some(fields) = refused {
            for hint in self.template_hints(fields.as_ref()) {
                if diag.has_hint_room() {
                    diag = diag.hint(hint);
                }
            }
        }
        diag
    }

    /// One hint per other template pinning a slot-filled field to the value
    /// the caller passed for it. `fields` holds the task fields the refusal is
    /// about ([`Refused::reads`]).
    fn template_hints(&self, fields: Option<&BTreeSet<String>>) -> Vec<Hint> {
        let Some(slotted) = slotted_fields(self.sources, self.explicit, fields)
            .filter(|slotted| !slotted.is_empty())
        else {
            return Vec::new();
        };
        list_templates(self.config, self.root)
            .into_iter()
            .filter(|name| name != self.template)
            .filter_map(|name| {
                let source = resolve_template(self.config, self.root, &name).ok()?;
                pins_any(&source, &slotted).then(|| {
                    Hint::command(
                        "file it with the template that sets that field itself",
                        Command::mt().word("add").word("-t").arg(&name),
                    )
                })
            })
            .collect()
    }
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

/// Whether `source` writes any slotted field as that same literal.
fn pins_any(source: &str, slotted: &[Slotted]) -> bool {
    let Ok(call) = task_with(source) else {
        return false;
    };
    slotted.iter().any(|(field, text, _)| {
        call.named(field)
            .is_some_and(|arg| source[arg.value_range.clone()] == *text)
    })
}

/// `--describe` takes its format after `=`, so a word beside it parses as the
/// title and the default template gets described without a word. Refuse it,
/// naming the template when the words spell one.
pub(super) fn refuse_title_beside_describe(
    title: &[String],
    config: &Config,
    root: &Path,
) -> Result<(), Diag> {
    if title.is_empty() {
        return Ok(());
    }
    let word = title.join(" ");
    let diag = Diag::usage("--describe takes no title").value(word.as_str());
    if list_templates(config, root).contains(&word) {
        return Err(diag.hint(Hint::command(
            "to describe that template, name it with -t",
            Command::mt()
                .word("add")
                .word("-t")
                .arg(&word)
                .word("--describe"),
        )));
    }
    Err(diag.hint(Hint::new("to describe a template, name it with -t")))
}
