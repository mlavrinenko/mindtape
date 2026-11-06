//! The bare `mt explain` index: the rules grouped by section, the derived
//! fields, and the tags, views and dictionaries this project declares, with no
//! messages or descriptions.

use mindtape::{Config, RuleSection, active_derived, active_rules, load_config};

use crate::diag::{Command, Result, next_line};
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

/// The width to wrap at when no terminal size is known.
const FALLBACK_WIDTH: usize = 100;

/// Everything the index lists, in the order it prints.
#[derive(Debug, PartialEq, Eq)]
struct Index {
    rules: Vec<(RuleSection, Vec<String>)>,
    derived: Vec<String>,
    tags: Vec<String>,
    views: Vec<String>,
    dicts: Vec<(String, usize)>,
}

/// Run `mt explain`.
///
/// # Errors
/// Returns an error if no project is found or its config fails to load.
pub fn run(as_json: bool) -> Result<()> {
    let config = load_config(&util::resolve_root()?)?;
    let index = collect(&config);
    if as_json {
        println!("{}", serde_json::to_string_pretty(&json(&index))?);
    } else {
        print!("{}", render(&index, width()));
    }
    Ok(())
}

/// Read the index off the merged config: every active rule grouped by its
/// section, every derived field, and the tag, view and dictionary names.
fn collect(config: &Config) -> Index {
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
        rules,
        derived,
        tags,
        views,
        dicts,
    }
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
/// view leads the list a reader reaches for first; every other order stands.
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
    out.push_str(&next_line(format!(
        "run {} for your work, or mt <command> --help for flags",
        Command::mt().word("ls")
    )));
    out.push('\n');
    out
}

/// `<label>: a, b, c`, breaking after a comma and indenting the rest by
/// `continuation` when the next entry would pass `width`. An empty list prints
/// `<label>: none`.
fn labelled(out: &mut String, label: &str, items: &[String], width: usize, continuation: &str) {
    if items.is_empty() {
        out.push_str(&format!("{label}: none\n"));
        return;
    }
    let prefix = format!("{label}: ");
    out.push_str(&prefix);
    let mut column = prefix.chars().count();
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
    out.push('\n');
}

/// The same index as `render`, grouped under a `schema_version` envelope.
fn json(index: &Index) -> serde_json::Value {
    let rules: serde_json::Map<String, serde_json::Value> = index
        .rules
        .iter()
        .map(|(section, names)| (section.path().to_owned(), serde_json::json!(names)))
        .collect();
    let dicts: serde_json::Map<String, serde_json::Value> = index
        .dicts
        .iter()
        .map(|(name, count)| (name.clone(), serde_json::json!(count)))
        .collect();
    serde_json::json!({
        "schema_version": mindtape::SCHEMA_VERSION,
        "rules": rules,
        "derived": index.derived,
        "tags": index.tags,
        "views": index.views,
        "dicts": dicts,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use mindtape::parse_config;

    use super::{Index, bare_name, collect, default_first, json, render};
    use mindtape::RuleSection;

    fn config() -> mindtape::Config {
        parse_config(
            "[tasks.tags]\nallowed = [\"cli\", \"default\", \"mt\"]\n\
             [[tasks.rules]]\nname = \"verb-title\"\nassert = 'has(title)'\n\
             [[tasks.rules]]\nassert = 'has(title)'\n\
             [[tasks.body.content.rules]]\nname = \"as-is\"\npath = [\"AS-IS\"]\nassert = \"non-empty\"\n\
             [[commits.rules]]\nname = \"names-a-task\"\nassert = 'has(refs)'\n\
             [[on.flip]]\nname = \"stale-wip\"\nwhen = 'into(\"wip\")'\n\
             actions = [{ list = \"@wip\", message = \"already in flight\" }, \
             { run = \"echo hi\", message = \"and this one too\" }]\n\
             [[tasks.collectors]]\nname = \"changes\"\nreturns = [\"path\"]\n\
             query = { section = \"Changes\", pattern = \"`([^`]+)`\" }\n\
             [views.default]\nfilter = \"\"\n\
             [views.blocked]\nfilter = \"blocked\"\n\
             [dicts.source]\nvalues = [\"a\", \"b\"]\n",
        )
        .unwrap()
    }

    #[test]
    fn groups_rules_by_section_and_drops_a_section_with_none() {
        let index = collect(&config());
        let sections: Vec<&str> = index.rules.iter().map(|(s, _)| s.path()).collect();
        assert_eq!(
            sections,
            [
                "tasks.rules",
                "tasks.body.content.rules",
                "commits.rules",
                "on.flip"
            ]
        );
        assert_eq!(index.rules[0].1, ["verb-title", "[1]"]);
        assert_eq!(index.rules[3].1, ["stale-wip"]);
    }

    #[test]
    fn bare_name_drops_the_section_and_an_action_slot() {
        assert_eq!(
            bare_name(RuleSection::TasksRules, "tasks.rules.verb-title"),
            "verb-title"
        );
        assert_eq!(bare_name(RuleSection::TasksRules, "tasks.rules[2]"), "[2]");
        assert_eq!(
            bare_name(RuleSection::OnFlip, "on.flip.stale-wip.actions[1]"),
            "stale-wip"
        );
        assert_eq!(
            bare_name(RuleSection::OnFlip, "on.flip[0].actions[0]"),
            "[0]"
        );
    }

    #[test]
    fn lists_derived_fields_without_the_prefix() {
        assert_eq!(collect(&config()).derived, ["changes"]);
    }

    #[test]
    fn puts_default_first_and_keeps_the_rest_in_order() {
        let index = collect(&config());
        assert_eq!(index.tags, ["default", "cli", "mt"]);
        assert_eq!(index.views, ["default", "blocked"]);
        assert_eq!(index.dicts[0].0, "mindtape-dev-verbs");
        assert!(index.dicts[0].1 > 0);
        assert_eq!(index.dicts[1].0, "mindtape-verbs");
        assert_eq!(index.dicts[2], ("source".to_owned(), 2));
        assert_eq!(
            default_first(vec!["b", "default", "a"], |s| s),
            ["default", "b", "a"]
        );
    }

    #[test]
    fn renders_every_group_and_wraps_a_long_list() {
        let index = Index {
            rules: vec![(RuleSection::OnFlip, vec!["stale-wip".to_owned()])],
            derived: vec!["changes".to_owned()],
            tags: vec!["cli".to_owned(), "default".to_owned(), "mt".to_owned()],
            views: vec!["default".to_owned()],
            dicts: vec![("source".to_owned(), 16)],
        };
        let text = render(&index, 24);
        assert_eq!(
            text,
            "\
rules:
  on.flip: stale-wip
derived fields: changes
tags: cli, default, mt
views: default
dicts: source (16)
next: run mt ls for your work, or mt <command> --help for flags
"
        );
    }

    #[test]
    fn wraps_after_a_comma_when_the_next_entry_passes_the_width() {
        let index = Index {
            rules: Vec::new(),
            derived: Vec::new(),
            tags: vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()],
            views: Vec::new(),
            dicts: Vec::new(),
        };
        assert_eq!(
            render(&index, 18),
            "\
rules: none
derived fields: none
tags: alpha, beta,
  gamma
views: none
dicts: none
next: run mt ls for your work, or mt <command> --help for flags
"
        );
    }

    #[test]
    fn the_index_ends_on_the_command_to_run_next() {
        let text = render(&collect(&config()), 100);
        assert!(
            text.ends_with("next: run mt ls for your work, or mt <command> --help for flags\n"),
            "{text}"
        );
    }

    #[test]
    fn json_carries_the_same_grouping() {
        let value = json(&collect(&config()));
        assert_eq!(value["schema_version"], mindtape::SCHEMA_VERSION);
        assert_eq!(
            value["rules"]["tasks.rules"],
            serde_json::json!(["verb-title", "[1]"])
        );
        assert_eq!(value["derived"], serde_json::json!(["changes"]));
        assert_eq!(value["tags"], serde_json::json!(["default", "cli", "mt"]));
        assert_eq!(value["views"], serde_json::json!(["default", "blocked"]));
        assert_eq!(value["dicts"]["source"], 2);
    }
}
