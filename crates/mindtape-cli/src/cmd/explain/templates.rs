//! `mt explain templates` lists every template `mt add` accepts, one line
//! each: its name, the `kind` it sets, how many slots it declares, and which
//! one `mt add` picks when the caller names none.
//!
//! The listing reads the same template files `mt add` reads, so a template
//! added to a configured directory appears without a config change.

use std::path::Path;

use mindtape::{Config, load_config};
use serde::Serialize;

use crate::cmd::add::values::{list_templates, resolve_template};
use crate::cmd::explain::emit;
use crate::cmd::schema::EXPLAIN_TEMPLATES;
use crate::diag::Result;
use crate::render::widest;
use crate::task_edit::task_with;
use crate::util;

/// The `kind` a template shows when it leaves the field to its caller.
const ANY_KIND: &str = "any";

/// One listed template.
#[derive(Debug, PartialEq, Eq)]
struct Template {
    /// The template's stem, as `mt add -t` takes it.
    name: String,
    /// The `kind` it sets, or [`ANY_KIND`] when it leaves the field to a slot.
    kind: String,
    /// How many `slot(...)` calls it declares.
    slots: usize,
    /// Whether `mt add` picks it when the caller names none.
    default: bool,
}

/// The JSON shape of one listed template.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct TemplateJson {
    /// The template's stem, as `mt add -t` takes it.
    pub name: String,
    /// The `kind` it sets, or `any` when it leaves the field to its caller.
    pub kind: String,
    /// How many `slot(...)` calls it declares.
    pub slots: usize,
    /// Whether `mt add` picks it when the caller names none.
    pub default: bool,
}

/// The JSON shape of `mt explain templates --json`.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct TemplatesPayload {
    /// Every template, in name order.
    pub templates: Vec<TemplateJson>,
}

/// Run `mt explain templates`.
///
/// # Errors
/// Returns an error if the project root does not resolve, its config fails to
/// load, or a template file fails to read.
pub fn run(as_json: bool) -> Result<()> {
    let root = util::resolve_root()?;
    let config = load_config(&root)?;
    print(&config, &root, as_json)
}

/// Print the listing for `config`'s templates, rooted at `root`.
///
/// # Errors
/// Returns an error when a template file fails to read.
pub(crate) fn print(config: &Config, root: &Path, as_json: bool) -> Result<()> {
    let templates = collect(config, root)?;
    let payload = TemplatesPayload {
        templates: templates.iter().map(template_json).collect(),
    };
    emit(as_json, EXPLAIN_TEMPLATES, payload, || render(&templates))
}

/// Every template `mt add` accepts, in name order, with the `kind` it sets and
/// the slots it declares.
fn collect(config: &Config, root: &Path) -> Result<Vec<Template>> {
    let default = config.templates.default.as_str();
    list_templates(config, root)
        .into_iter()
        .map(|name| {
            let source = resolve_template(config, root, &name)?;
            Ok(Template {
                kind: kind_of(&source),
                slots: mindtape::discover_slots(&source).len(),
                default: name == default,
                name,
            })
        })
        .collect()
}

/// The `kind` a template sets, or [`ANY_KIND`] when it leaves the field to a
/// slot or omits it.
fn kind_of(source: &str) -> String {
    let literal = task_with(source).ok().and_then(|call| {
        let range = call.named("kind")?.value_range.clone();
        let value = source.get(range)?.trim();
        Some(value.strip_prefix('"')?.strip_suffix('"')?.to_owned())
    });
    literal.unwrap_or_else(|| ANY_KIND.to_owned())
}

fn template_json(template: &Template) -> TemplateJson {
    TemplateJson {
        name: template.name.clone(),
        kind: template.kind.clone(),
        slots: template.slots,
        default: template.default,
    }
}

/// One row per template, aligned: name, `kind`, slot count, and `default` on
/// the one `mt add` picks.
fn render(templates: &[Template]) -> String {
    if templates.is_empty() {
        return "no templates\n".to_owned();
    }
    let name_width = widest(templates.iter().map(|template| template.name.as_str()));
    let kind_width = widest(templates.iter().map(|template| template.kind.as_str()));
    let rows: Vec<(String, bool)> = templates
        .iter()
        .map(|template| {
            let row = format!(
                "{:<name_width$}  kind: {:<kind_width$}  {}",
                template.name,
                template.kind,
                mindtape::count(template.slots, "slot"),
            );
            (row, template.default)
        })
        .collect();
    let row_width = widest(rows.iter().map(|(row, _)| row.as_str()));
    let mut out = String::new();
    for (row, default) in rows {
        if default {
            out.push_str(&format!("{row:<row_width$}   default\n"));
        } else {
            out.push_str(&format!("{row}\n"));
        }
    }
    out
}

#[cfg(test)]
#[path = "templates_tests.rs"]
mod tests;
