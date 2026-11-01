//! `mt tags` — list the allowed tags and the rules that read them.
//!
//! Read-only. Each allowed tag is followed by every rule whose `when` or
//! `assert` mentions `has_tag("<tag>")`, and by that rule's message. A tag no
//! rule reads prints as an area label, so the listing covers the whole
//! vocabulary. `--for` keeps the rules the command at one moment enforces.

use mindtape::{ActiveRule, Config, active_rules, load_config};

use crate::cmd::rules::{Moment, message};
use crate::diag::Result;
use crate::render::widest;
use crate::util;

/// Arguments for `tags`.
#[derive(Debug, clap::Args)]
pub struct TagsArgs {
    /// List only the tags rules enforce at this moment.
    #[arg(long = "for", value_name = "MOMENT", value_enum)]
    pub moment: Option<Moment>,

    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// One allowed tag and the rules that read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagEntry {
    pub(crate) tag: String,
    pub(crate) rules: Vec<TagRule>,
}

/// A rule that reads a tag, as the listing shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagRule {
    pub(crate) name: String,
    pub(crate) message: String,
}

/// Run the `tags` command.
///
/// # Errors
/// Returns an error if no project is found or its config fails to load.
pub fn run(args: &TagsArgs) -> Result<()> {
    let config = load_config(&util::resolve_root()?)?;
    let entries = entries(&config, args.moment);
    if args.json {
        println!("{}", serde_json::to_string_pretty(&json(&entries))?);
    } else {
        print!("{}", render(&entries));
    }
    Ok(())
}

/// Every allowed tag in config order, each with the rules that read it.
fn entries(config: &Config, moment: Option<Moment>) -> Vec<TagEntry> {
    let rules: Vec<ActiveRule> = active_rules(config)
        .into_iter()
        .filter(|rule| moment.is_none_or(|moment| moment.enforces(rule.section)))
        .collect();
    config
        .tasks
        .tags
        .allowed
        .iter()
        .map(|tag| TagEntry {
            tag: tag.clone(),
            rules: rules
                .iter()
                .filter(|rule| reads(rule, tag))
                .map(tag_rule)
                .collect(),
        })
        .collect()
}

/// Whether a rule's `when` or `assert` names `tag` as `has_tag("<tag>")`.
fn reads(rule: &ActiveRule, tag: &str) -> bool {
    let needle = format!("has_tag(\"{tag}\")");
    [&rule.when, &rule.assertion]
        .into_iter()
        .flatten()
        .any(|text| text.contains(&needle))
}

/// One rule as the tag listing shows it.
fn tag_rule(rule: &ActiveRule) -> TagRule {
    TagRule {
        name: rule.name.clone(),
        message: message(rule),
    }
}

/// `<tag>  <rule>` with the message under the rule, or `<tag>  area label`
/// when no rule reads it.
fn render(entries: &[TagEntry]) -> String {
    let width = widest(entries.iter().map(|entry| entry.tag.as_str()));
    let mut out = String::new();
    for entry in entries {
        if entry.rules.is_empty() {
            out.push_str(&format!("{:<width$}  area label\n", entry.tag));
            continue;
        }
        for rule in &entry.rules {
            out.push_str(&format!("{:<width$}  {}\n", entry.tag, rule.name));
            out.push_str(&format!("{:width$}  {}\n", "", rule.message));
        }
    }
    if out.is_empty() {
        out.push_str("no tag list; every tag is allowed\n");
    }
    out
}

/// The same map as `render`, under a `schema_version` envelope.
fn json(entries: &[TagEntry]) -> serde_json::Value {
    let tags: serde_json::Map<String, serde_json::Value> = entries
        .iter()
        .map(|entry| {
            let rules: Vec<serde_json::Value> = entry
                .rules
                .iter()
                .map(|rule| {
                    serde_json::json!({
                        "rule": rule.name,
                        "message": rule.message,
                    })
                })
                .collect();
            (entry.tag.clone(), serde_json::Value::Array(rules))
        })
        .collect();
    serde_json::json!({
        "schema_version": mindtape::SCHEMA_VERSION,
        "tags": tags,
    })
}

#[cfg(test)]
#[path = "tags_tests.rs"]
mod tests;
