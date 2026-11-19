//! The bare `mt explain` index. It prints the project's shape, the rules under
//! the block each belongs to, the derived fields, and the tags, views and
//! dictionaries this project declares. No messages or descriptions.
//!
//! The shape blocks read the merged config alone. The index doesn't load a
//! task, count anything or evaluate a filter. It costs one config load and one
//! template-directory scan.

use std::path::Path;

use mindtape::explain::Shape;
use mindtape::{
    ActiveAddHook, ActiveRule, Config, RuleOriginEntry, RuleSection, Severity, active_add,
    active_derived, active_rules,
};

use crate::cmd::schema::EXPLAIN;
use crate::diag::Result;
use crate::render::envelope::Envelope;
use crate::render::objects::{ExplainIndexPayload, RefusalJson};

mod render;

use render::render;

/// The order the index prints rule groups in, the same order the config
/// declares them and `mt explain rules` lists them.
const SECTIONS: [RuleSection; 6] = [
    RuleSection::TasksRules,
    RuleSection::BodyHeaders,
    RuleSection::BodyContent,
    RuleSection::CommitsRules,
    RuleSection::OnFlip,
    RuleSection::OnRm,
];

/// The severities a section splits into, strictest first. A reader meets what
/// refuses the work before what only warns.
const SEVERITIES: [Severity; 2] = [Severity::Error, Severity::Warn];

/// One section's rule names, split by severity, strictest first.
type RuleGroups = Vec<(Severity, Vec<String>)>;

/// The width to wrap at when the terminal does not report a size.
const FALLBACK_WIDTH: usize = 100;

/// Everything the index lists, in the order it prints.
struct Index {
    shape: Shape,
    refusals: Vec<RefusalJson>,
    rules: Vec<(RuleSection, RuleGroups)>,
    add: Vec<String>,
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
    let (root, config, origins) = super::load()?;
    let index = collect(&config, &origins, &root);
    if as_json {
        println!("{}", serde_json::to_string_pretty(&document(&index))?);
    } else {
        print!("{}", render(&index, width()));
    }
    Ok(())
}

/// Read the index off the merged config. It holds the project's shape, every
/// active rule grouped by its section, and every add hook. It also holds every
/// derived field, and the tag, view and dictionary names.
fn collect(config: &Config, origins: &[RuleOriginEntry], root: &Path) -> Index {
    let shape = mindtape::explain::shape(config, root);
    let refusals = refusals(config, root);
    let active = active_rules(config);
    let rules = SECTIONS
        .iter()
        .map(|&section| {
            let groups: RuleGroups = SEVERITIES
                .iter()
                .filter_map(|&severity| {
                    let mut names: Vec<String> = active
                        .iter()
                        .filter(|rule| rule.section == section && rule.severity == severity)
                        .map(|rule| bare_name(section, rule))
                        .collect();
                    // A hook with several listed actions is still one hook in
                    // the index; `mt explain rules` is where its actions stay
                    // distinct.
                    names.dedup();
                    (!names.is_empty()).then_some((severity, names))
                })
                .collect();
            (section, groups)
        })
        .filter(|(_, groups): &(RuleSection, RuleGroups)| !groups.is_empty())
        .collect();
    let add = active_add(config, origins).iter().map(add_name).collect();
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
    let dicts: Vec<(String, usize)> = config
        .compiled_dicts()
        .iter()
        .map(|(name, dict)| (name.to_owned(), dict.len()))
        .collect();
    let dicts = default_first(dicts, |(name, _)| name);
    Index {
        shape,
        refusals,
        rules,
        add,
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

/// The hook's name, or for an unnamed hook the directory it files into, or
/// its `on.add[<index>]` label when it files nowhere.
fn add_name(hook: &ActiveAddHook<'_>) -> String {
    if let Some(name) = &hook.hook.name {
        return name.clone();
    }
    hook.hook
        .actions
        .iter()
        .find_map(|action| action.dir.as_ref().and_then(|dir| dir.template()))
        .map_or_else(|| hook.label.clone(), str::to_owned)
}

/// The rule's name without the section path that leads it: `verb-title` for
/// `tasks.rules.verb-title`, and the hook's name for one of its actions. An
/// unnamed hook has no name to print. Its action prints what it runs, else
/// the message it carries, else its `on.flip[<index>]` label.
fn bare_name(section: RuleSection, rule: &ActiveRule) -> String {
    let rest = rule.name.strip_prefix(section.path()).unwrap_or(&rule.name);
    let rest = rest.strip_prefix('.').unwrap_or(rest);
    if !matches!(section, RuleSection::OnFlip | RuleSection::OnRm) {
        return rest.to_owned();
    }
    let hook = rest.split(".actions[").next().unwrap_or(rest);
    if !hook.starts_with('[') {
        return hook.to_owned();
    }
    rule.run
        .clone()
        .or_else(|| rule.message.clone())
        .unwrap_or_else(|| format!("{}{hook}", section.path()))
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

/// The same index as `render`, inside its `$schema` envelope.
fn document(index: &Index) -> Envelope<ExplainIndexPayload> {
    let rules = index
        .rules
        .iter()
        .map(|(section, groups)| {
            let groups = groups
                .iter()
                .map(|(severity, names)| (severity.label().to_owned(), names.clone()))
                .collect();
            (section.path().to_owned(), groups)
        })
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
            add: index.add.clone(),
            derived: index.derived.clone(),
            tags: index.tags.clone(),
            views: index.views.clone(),
            dicts,
        },
    )
}

#[cfg(test)]
#[path = "../index_tests.rs"]
mod tests;
