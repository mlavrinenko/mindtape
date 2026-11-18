//! The bare `mt explain` index: the project's shape, the rules grouped by
//! section, the derived fields, and the tags, views and dictionaries this
//! project declares, with no messages or descriptions.
//!
//! The shape blocks read the merged config alone. The index doesn't load a
//! task, count anything or evaluate a filter. It costs one config load and one
//! template-directory scan.

use std::path::Path;

use mindtape::explain::{ProjectShape, Shape, TaskShape};
use mindtape::{Config, RuleSection, active_derived, active_rules, load_config};

use crate::cmd::schema::EXPLAIN;
use crate::diag::{Command, Result, next_line};
use crate::render::envelope::Envelope;
use crate::render::objects::{ExplainIndexPayload, RefusalJson};
use crate::util;

/// The order the index prints rule groups in, the same order the config
/// declares them and `mt explain rules` lists them.
const SECTIONS: [RuleSection; 5] = [
    RuleSection::TasksRules,
    RuleSection::BodyHeaders,
    RuleSection::BodyContent,
    RuleSection::CommitsRules,
    RuleSection::OnFlip,
];

/// The width to wrap at when the terminal does not report a size.
const FALLBACK_WIDTH: usize = 100;

/// Everything the index lists, in the order it prints.
struct Index {
    shape: Shape,
    refusals: Vec<RefusalJson>,
    rules: Vec<(RuleSection, Vec<String>)>,
    derived: Vec<String>,
    tags: Vec<String>,
    views: Vec<String>,
    dicts: Vec<(String, usize)>,
}

/// Run `mt explain`.
///
/// # Errors
/// Returns an error if the project root does not resolve or its config fails to load.
pub fn run(as_json: bool) -> Result<()> {
    let root = util::resolve_root()?;
    let config = load_config(&root)?;
    let index = collect(&config, &root);
    if as_json {
        println!("{}", serde_json::to_string_pretty(&document(&index))?);
    } else {
        print!("{}", render(&index, width()));
    }
    Ok(())
}

/// Read the index off the merged config: the project's shape, every active
/// rule grouped by its section, every derived field, and the tag, view and
/// dictionary names.
fn collect(config: &Config, root: &Path) -> Index {
    let shape = mindtape::explain::shape(config, root);
    let refusals = refusals(config, root);
    let active = active_rules(config);
    let rules = SECTIONS
        .iter()
        .map(|&section| {
            let mut names: Vec<String> = active
                .iter()
                .filter(|rule| rule.section == section)
                .map(|rule| bare_name(section, &rule.name))
                .collect();
            // A hook with several listed actions is still one hook in the
            // index; `mt explain rules` is where its actions stay distinct.
            names.dedup();
            (section, names)
        })
        .filter(|(_, names): &(RuleSection, Vec<String>)| !names.is_empty())
        .collect();
    let derived = active_derived(config)
        .into_iter()
        .map(|field| {
            field
                .name
                .strip_prefix("derived.")
                .unwrap_or(&field.name)
                .to_owned()
        })
        .collect();
    let tags = default_first(config.tasks.tags.allowed.clone(), String::as_str);
    let views = default_first(config.views.keys().cloned().collect(), String::as_str);
    let dicts = default_first(
        config
            .compiled_dicts()
            .iter()
            .map(|(name, dict)| (name.to_owned(), dict.len()))
            .collect(),
        |(name, _)| name,
    );
    Index {
        shape,
        refusals,
        rules,
        derived,
        tags,
        views,
        dicts,
    }
}

/// The gate commands the user has not allowed, as the `refusals:` block and
/// its JSON show them.
fn refusals(config: &Config, root: &Path) -> Vec<RefusalJson> {
    mindtape::allow::refused_gates(root, config)
        .into_iter()
        .map(|gate| RefusalJson {
            gate: gate.gate.to_owned(),
            command: gate.command.to_string(),
        })
        .collect()
}

/// The rule's name without the section path that leads it: `verb-title` for
/// `tasks.rules.verb-title`, and the hook's name for one of its actions.
fn bare_name(section: RuleSection, name: &str) -> String {
    let rest = name.strip_prefix(section.path()).unwrap_or(name);
    let rest = rest.strip_prefix('.').unwrap_or(rest);
    if section == RuleSection::OnFlip {
        rest.split(".actions[").next().unwrap_or(rest).to_owned()
    } else {
        rest.to_owned()
    }
}

/// `items` with any entry called `default` moved to the front. The default
/// view leads the list a reader reaches for first, and every other order stands.
fn default_first<T>(items: Vec<T>, name: impl Fn(&T) -> &str) -> Vec<T> {
    let (mut first, rest): (Vec<T>, Vec<T>) =
        items.into_iter().partition(|item| name(item) == "default");
    first.extend(rest);
    first
}

/// `Some(width)` from the terminal, or [`FALLBACK_WIDTH`] when none answers.
fn width() -> usize {
    terminal_size::terminal_size().map_or(FALLBACK_WIDTH, |(width, _)| usize::from(width.0))
}

/// The index as text, one group per line and comma lists wrapped at `width`.
fn render(index: &Index, width: usize) -> String {
    let mut out = String::new();
    render_project(&mut out, &index.shape.project, width);
    render_task(&mut out, &index.shape.task, width);
    out.push_str("commits:\n");
    labelled(
        &mut out,
        "  trailer",
        std::slice::from_ref(&index.shape.commits.trailer),
        width,
        "    ",
    );
    if index.shape.absent.is_empty() {
        out.push_str("absent: none\n");
    } else {
        labelled(&mut out, "absent", &index.shape.absent, width, "  ");
    }
    render_refusals(&mut out, &index.refusals);
    if index.rules.is_empty() {
        out.push_str("rules: none\n");
    } else {
        out.push_str("rules:\n");
        for (section, names) in &index.rules {
            let label = format!("  {}", section.path());
            labelled(&mut out, &label, names, width, "  ");
        }
    }
    labelled(&mut out, "derived fields", &index.derived, width, "  ");
    if index.tags.is_empty() {
        out.push_str("tags: any\n");
    } else {
        labelled(&mut out, "tags", &index.tags, width, "  ");
    }
    labelled(&mut out, "views", &index.views, width, "  ");
    let dicts: Vec<String> = index
        .dicts
        .iter()
        .map(|(name, count)| format!("{name} ({count})"))
        .collect();
    labelled(&mut out, "dicts", &dicts, width, "  ");
    out.push('\n');
    out.push_str(&next_line(format!(
        "{} [<add|rules|tags|views|dicts>] [--json]",
        Command::mt().word("explain")
    )));
    out.push('\n');
    out
}

/// The `project:` block: one line per key, and the status legend's two groups
/// under it.
fn render_project(out: &mut String, project: &ProjectShape, width: usize) {
    out.push_str("project:\n");
    let roots: Vec<String> = project
        .roots
        .iter()
        .map(|root| format!("{} (tag: {})", root.path, root.tag))
        .collect();
    labelled(out, "  roots", &roots, width, "    ");
    if let Some(statuses) = &project.statuses {
        out.push_str("  statuses:\n");
        labelled(out, "    open", &statuses.open, width, "      ");
        labelled(out, "    terminal", &statuses.terminal, width, "      ");
    }
    if !project.actors.is_empty() {
        let actors: Vec<String> = project
            .actors
            .iter()
            .map(|actor| {
                let mut parts = vec![actor.kind.clone()];
                parts.extend(actor.roles.iter().cloned());
                format!("{} ({})", actor.name, parts.join(", "))
            })
            .collect();
        labelled(out, "  actors", &actors, width, "    ");
    }
    if let Some(rm) = &project.rm {
        labelled(out, "  rm", &[format!("{rm} only")], width, "    ");
    }
}

/// The `task:` block: one line per field, and the templates' default last.
fn render_task(out: &mut String, task: &TaskShape, width: usize) {
    out.push_str("task:\n");
    let priority = match &task.priority.framework {
        Some(name) if !task.priority.inputs.is_empty() => {
            format!("{name} ({})", task.priority.inputs.join(", "))
        }
        Some(name) => name.clone(),
        None => bounds(task.priority.min, task.priority.max),
    };
    labelled(out, "  priority", &[priority], width, "    ");
    labelled(
        out,
        "  difficulty",
        &[bounds(task.difficulty.min, task.difficulty.max)],
        width,
        "    ",
    );
    if let Some(estimate) = &task.estimate {
        let required = match estimate.required.as_str() {
            "false" => "optional".to_owned(),
            "true" => "required".to_owned(),
            expr => format!("required: {expr}"),
        };
        let units = format!("{} ({required})", estimate.units.join(", "));
        labelled(out, "  estimate", &[units], width, "    ");
    }
    if !task.kinds.is_empty() {
        labelled(out, "  kinds", &task.kinds, width, "    ");
    }
    labelled(out, "  links", &task.links, width, "    ");
    if let Some(comments) = &task.comments {
        labelled(
            out,
            "  comments",
            std::slice::from_ref(comments),
            width,
            "    ",
        );
    }
    if !task.extensions.is_empty() {
        let extensions: Vec<String> = task
            .extensions
            .iter()
            .map(|extension| {
                if extension.rules.is_empty() {
                    extension.name.clone()
                } else {
                    format!("{} ({})", extension.name, extension.rules.join(", "))
                }
            })
            .collect();
        labelled(out, "  extensions", &extensions, width, "    ");
    }
    let mut templates = list_line("  templates", &task.templates.names, width, "    ");
    templates.push_str(&format!(" (default: {})", task.templates.default));
    out.push_str(&templates);
    out.push('\n');
}

/// The `refusals:` block: one line per gate command the user has not allowed,
/// each naming the command that records the approval.
fn render_refusals(out: &mut String, refusals: &[RefusalJson]) {
    if refusals.is_empty() {
        return;
    }
    out.push_str("refusals:\n");
    for refusal in refusals {
        out.push_str(&format!(
            "  {}: {} (not allowed; ask the user to run `{}`)\n",
            refusal.gate,
            refusal.command,
            Command::mt().word("allow")
        ));
    }
}

/// A scale's bounds as `0-9`.
fn bounds(min: f64, max: f64) -> String {
    format!("{min}-{max}")
}

/// `<label>: a, b, c`, breaking after a comma and indenting the rest by
/// `continuation` when the next entry would pass `width`. An empty list prints
/// `<label>: none`.
fn list_line(label: &str, items: &[String], width: usize, continuation: &str) -> String {
    if items.is_empty() {
        return format!("{label}: none");
    }
    let mut out = format!("{label}: ");
    let mut column = out.chars().count();
    for (index, item) in items.iter().enumerate() {
        let piece = if index == 0 {
            item.clone()
        } else {
            format!(", {item}")
        };
        let piece_width = piece.chars().count();
        if index > 0 && column + piece_width > width {
            out.push_str(",\n");
            out.push_str(continuation);
            out.push_str(item);
            column = continuation.chars().count() + item.chars().count();
        } else {
            out.push_str(&piece);
            column += piece_width;
        }
    }
    out
}

/// [`list_line`] with its newline.
fn labelled(out: &mut String, label: &str, items: &[String], width: usize, continuation: &str) {
    out.push_str(&list_line(label, items, width, continuation));
    out.push('\n');
}

/// The same index as `render`, inside its `$schema` envelope.
fn document(index: &Index) -> Envelope<ExplainIndexPayload> {
    let rules = index
        .rules
        .iter()
        .map(|(section, names)| (section.path().to_owned(), names.clone()))
        .collect();
    let dicts = index.dicts.iter().cloned().collect();
    Envelope::new(
        EXPLAIN,
        ExplainIndexPayload {
            project: index.shape.project.clone(),
            task: index.shape.task.clone(),
            commits: index.shape.commits.clone(),
            absent: index.shape.absent.clone(),
            refusals: index.refusals.clone(),
            rules,
            derived: index.derived.clone(),
            tags: index.tags.clone(),
            views: index.views.clone(),
            dicts,
        },
    )
}

#[cfg(test)]
#[path = "index_tests.rs"]
mod tests;
