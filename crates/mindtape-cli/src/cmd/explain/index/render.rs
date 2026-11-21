//! The bare `mt explain` index as text: one block per scope, one line per
//! group, and comma lists wrapped at the terminal's width.

use mindtape::RuleSection;
use mindtape::explain::{CommitsShape, InputShape, ProjectShape, TaskShape};

use crate::diag::{Command, more_line};

use super::{Index, Label, RuleGroups};

/// The index as text, one group per line and comma lists wrapped at `width`.
pub(super) fn render(index: &Index, width: usize) -> String {
    let mut out = String::new();
    render_project(&mut out, &index.shape.project, width);
    render_task(&mut out, index, width);
    render_commits(&mut out, &index.shape.commits, &index.rules, width);
    render_hooks(&mut out, index, width);
    labelled(&mut out, "derived fields", &index.derived, width);
    if index.tags.is_empty() {
        out.push_str("tags: any\n");
    } else {
        labelled(&mut out, "tags", &index.tags, width);
    }
    labelled(&mut out, "views", &index.views, width);
    let dicts: Vec<String> = index
        .dicts
        .iter()
        .map(|(name, count)| format!("{name} ({count})"))
        .collect();
    labelled(&mut out, "dicts", &dicts, width);
    out.push('\n');
    out.push_str(&more_line(format!(
        "{} <add|rules|tags|views|dicts>",
        Command::mt().word("explain")
    )));
    out.push('\n');
    if !index.blocked.is_empty() {
        out.push_str(&allow_line());
        out.push('\n');
    }
    out
}

/// The line that says how to record the approval a `not allowed` label asks
/// for. It prints once, under the `more:` line, so no block repeats it.
fn allow_line() -> String {
    format!(
        "allow: {} records the approval for each command marked not allowed",
        Command::mt().word("allow")
    )
}

/// The `project:` block: one line per key, and the status legend's two groups
/// under it. A key the project leaves open prints what its absence permits.
fn render_project(out: &mut String, project: &ProjectShape, width: usize) {
    out.push_str("project:\n");
    let roots: Vec<String> = project
        .roots
        .iter()
        .map(|root| format!("{} (tag: {})", root.path, root.tag))
        .collect();
    labelled(out, "  roots", &roots, width);
    match &project.statuses {
        Some(statuses) => {
            out.push_str("  statuses:\n");
            labelled(out, "    open", &statuses.open, width);
            labelled(out, "    terminal", &statuses.terminal, width);
        }
        None => out.push_str("  statuses: any\n"),
    }
    let actors: Vec<String> = project
        .actors
        .iter()
        .map(|actor| {
            let mut parts = vec![actor.kind.clone()];
            parts.extend(actor.roles.iter().cloned());
            labelled_parts(&actor.name, &parts)
        })
        .collect();
    labelled_or(out, "  actors", &actors, "none", width);
}

/// The `task:` block: one line per field, the templates' default last, the
/// rules that check a task under them, and the `[[tasks.waits]]` checkers.
fn render_task(out: &mut String, index: &Index, width: usize) {
    out.push_str("task:\n");
    render_task_fields(out, &index.shape.task, width);
    let rules = &index.rules;
    render_section(out, rules, RuleSection::TasksRules, "  rules", width);
    render_section(
        out,
        rules,
        RuleSection::BodyHeaders,
        "  body.headers",
        width,
    );
    render_section(
        out,
        rules,
        RuleSection::BodyContent,
        "  body.content",
        width,
    );
    let (blocked, allowed): (Vec<String>, Vec<String>) = index
        .waits
        .iter()
        .cloned()
        .partition(|wait| index.blocked.contains(wait));
    if !allowed.is_empty() {
        labelled(out, "  waits", &allowed, width);
    }
    if !blocked.is_empty() {
        labelled(out, "  waits (not allowed)", &blocked, width);
    }
}

/// The `task:` block's field lines, from the priority scale to the templates.
fn render_task_fields(out: &mut String, task: &TaskShape, width: usize) {
    let priority = match &task.priority.framework {
        Some(name) if !task.priority.inputs.is_empty() => {
            let inputs: Vec<String> = task.priority.inputs.iter().map(input_line).collect();
            format!(
                "{name} ({}) -> {}",
                inputs.join(", "),
                bounds(task.priority.min, task.priority.max)
            )
        }
        Some(name) => name.clone(),
        None => bounds(task.priority.min, task.priority.max),
    };
    labelled(out, "  priority", &[priority], width);
    labelled(
        out,
        "  difficulty",
        &[bounds(task.difficulty.min, task.difficulty.max)],
        width,
    );
    match &task.estimate {
        Some(estimate) => {
            let required = match estimate.required.as_str() {
                "false" => "optional".to_owned(),
                "true" => "required".to_owned(),
                expr => format!("required: {expr}"),
            };
            let units = format!("{} ({required})", estimate.units.join(", "));
            labelled(out, "  estimate", &[units], width);
        }
        None => out.push_str("  estimate: any\n"),
    }
    labelled_or(out, "  kinds", &task.kinds, "any", width);
    labelled(out, "  links", &task.links, width);
    match &task.comments {
        Some(comments) => labelled(out, "  comments", std::slice::from_ref(comments), width),
        None => out.push_str("  comments: allowed\n"),
    }
    let extensions: Vec<String> = task
        .extensions
        .iter()
        .map(|extension| {
            if extension.rules.is_empty() {
                extension.name.clone()
            } else {
                labelled_parts(&extension.name, &extension.rules)
            }
        })
        .collect();
    labelled_or(out, "  extensions", &extensions, "none", width);
    let mut templates = list_line("  templates", &task.templates.names, width);
    templates.push_str(&format!(" (default: {})", task.templates.default));
    out.push_str(&templates);
    out.push('\n');
}

/// The `commits:` block: the trailer, and the rules that check a commit.
fn render_commits(
    out: &mut String,
    commits: &CommitsShape,
    rules: &[(RuleSection, RuleGroups)],
    width: usize,
) {
    out.push_str("commits:\n");
    labelled(
        out,
        "  trailer",
        std::slice::from_ref(&commits.trailer),
        width,
    );
    render_section(out, rules, RuleSection::CommitsRules, "  rules", width);
}

/// The `add:`, `flip:` and `rm:` blocks: what each write command runs, and the
/// `[rm] default-filter` beside the removal hooks it governs.
fn render_hooks(out: &mut String, index: &Index, width: usize) {
    if !index.add.is_empty() {
        out.push_str("add:\n");
        labelled(out, "  hooks", &index.add, width);
    }
    if !groups(&index.rules, RuleSection::OnFlip).is_empty() {
        out.push_str("flip:\n");
        render_section(out, &index.rules, RuleSection::OnFlip, "  hooks", width);
    }
    let rm = index.shape.project.rm.as_ref();
    if rm.is_some() || !groups(&index.rules, RuleSection::OnRm).is_empty() {
        out.push_str("rm:\n");
        if let Some(filter) = rm {
            labelled(out, "  filter", std::slice::from_ref(filter), width);
        }
        render_section(out, &index.rules, RuleSection::OnRm, "  hooks", width);
    }
}

/// One section's rules, one line per severity, under `label` at the block's
/// own indent.
fn render_section(
    out: &mut String,
    rules: &[(RuleSection, RuleGroups)],
    section: RuleSection,
    label: &str,
    width: usize,
) {
    for (meaning, names) in groups(rules, section) {
        let line = format!("{label} ({})", meaning.text());
        labelled(out, &line, names, width);
    }
}

/// The label groups one section holds, empty when it holds none.
fn groups(rules: &[(RuleSection, RuleGroups)], section: RuleSection) -> &[(Label, Vec<String>)] {
    rules
        .iter()
        .find(|(held, _)| *held == section)
        .map_or(&[], |(_, groups)| groups.as_slice())
}

/// A scale's bounds as `0-9`.
fn bounds(min: f64, max: f64) -> String {
    format!("{min}-{max}")
}

/// `name (a, b)`: a label with its parenthetical parts.
fn labelled_parts(name: &str, parts: &[String]) -> String {
    format!("{name} ({})", parts.join(", "))
}

/// One framework input: `impact 1-10` for a number, `urgent` for a bool, and
/// `level: must|should` for an enum. The pipe keeps an enum's values out of the
/// comma list that separates the inputs.
pub(super) fn input_line(input: &InputShape) -> String {
    match input.kind.as_str() {
        "number" => match (input.min, input.max) {
            (Some(min), Some(max)) => format!("{} {}", input.name, bounds(min, max)),
            _ => input.name.clone(),
        },
        "enum" => format!("{}: {}", input.name, input.values.join("|")),
        _ => input.name.clone(),
    }
}

/// The indent a wrapped entry continues at: the label's own indent plus two.
/// A continuation never lands in the column of a section label.
fn continuation(label: &str) -> String {
    let indent = label.chars().take_while(|ch| *ch == ' ').count();
    " ".repeat(indent + 2)
}

/// `<label>: a, b, c`, breaking after a comma and indenting the rest under the
/// label when the next entry would pass `width`. An empty list prints
/// `<label>: none`.
pub(super) fn list_line(label: &str, items: &[String], width: usize) -> String {
    if items.is_empty() {
        return format!("{label}: none");
    }
    let continuation = continuation(label);
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
            out.push_str(&continuation);
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
fn labelled(out: &mut String, label: &str, items: &[String], width: usize) {
    out.push_str(&list_line(label, items, width));
    out.push('\n');
}

/// [`labelled`], or `empty` when the project declares none: `actors: none`,
/// `kinds: any`.
fn labelled_or(out: &mut String, label: &str, items: &[String], empty: &str, width: usize) {
    if items.is_empty() {
        out.push_str(&format!("{label}: {empty}\n"));
    } else {
        labelled(out, label, items, width);
    }
}
