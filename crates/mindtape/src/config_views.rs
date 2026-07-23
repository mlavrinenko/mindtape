//! `[views]` config: named query bundles.
//!
//! A view names a filter plus optional sort and column set. It is referenced
//! from the CLI by an `@name` sigil and from the filter language by
//! `in_view("name")`. Validated at config load ([`validate_views`]): each filter
//! parses, sort specs and columns are legal, every `in_view` / `@` reference
//! resolves to a declared view, and the reference graph is acyclic.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config_ls::LS_COLUMNS;
use crate::filter::Filter;
use crate::sort::validate_sort_spec;

/// `[views.<name>]` — one named query bundle.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ViewSpec {
    /// Filter expression (the filter facet). May reference other views via
    /// `in_view("other")` or the `@other` sugar. Empty means match-all.
    pub filter: String,
    /// Sort chain applied when this view drives sort (bundle mode or
    /// `--sort @name`). Same spec grammar as `--sort`; empty means the default
    /// order.
    pub sort: Vec<String>,
    /// Column set applied when this view drives columns (bundle mode or
    /// `--columns @name`). Empty means the default column set.
    pub columns: Vec<String>,
    /// Human description, shown by `mt views`.
    pub description: Option<String>,
}

/// Validate every `[views]` entry: filter parses, sort specs and columns are
/// legal, each `in_view` / `@` reference resolves, and the reference graph has
/// no cycle. Returns one message per problem.
#[must_use]
pub fn validate_views(views: &BTreeMap<String, ViewSpec>) -> Vec<String> {
    let mut problems = Vec::new();
    let legal_cols = LS_COLUMNS.join(", ");
    let mut refs: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for (name, spec) in views {
        match Filter::parse(&spec.filter) {
            Ok(filter) => {
                let named = filter.view_refs();
                for target in &named {
                    if !views.contains_key(target) {
                        problems.push(format!("views.{name}: unknown view reference @{target}"));
                    }
                }
                refs.insert(name.as_str(), named);
            }
            Err(err) => problems.push(format!("views.{name}.filter: {err}")),
        }
        for spec_str in &spec.sort {
            if let Err(msg) = validate_sort_spec(spec_str) {
                problems.push(format!("views.{name}.sort: {msg}"));
            }
        }
        for col in &spec.columns {
            if !LS_COLUMNS.contains(&col.as_str()) {
                problems.push(format!(
                    "views.{name}.columns: unknown column `{col}`, expected one of {legal_cols}"
                ));
            }
        }
    }
    if let Some(cycle) = find_cycle(&refs) {
        problems.push(format!("views: reference cycle {}", cycle.join(" → ")));
    }
    problems
}

/// State marks for the cycle-detection DFS.
const UNSEEN: u8 = 0;
const ACTIVE: u8 = 1;
const DONE: u8 = 2;

/// Depth-first search for a cycle in the view-reference graph. Returns the names
/// on the first cycle found (closing back to the first), or `None` if acyclic.
/// Edges to a target absent from `refs` (an unknown reference, reported
/// separately) are ignored — they cannot join a cycle.
fn find_cycle(refs: &BTreeMap<&str, Vec<String>>) -> Option<Vec<String>> {
    let mut state: BTreeMap<String, u8> = BTreeMap::new();
    for start in refs.keys() {
        let mut stack: Vec<String> = Vec::new();
        if let Some(cycle) = dfs(start, refs, &mut state, &mut stack) {
            return Some(cycle);
        }
    }
    None
}

fn dfs(
    node: &str,
    refs: &BTreeMap<&str, Vec<String>>,
    state: &mut BTreeMap<String, u8>,
    stack: &mut Vec<String>,
) -> Option<Vec<String>> {
    match state.get(node).copied().unwrap_or(UNSEEN) {
        DONE => return None,
        ACTIVE => {
            let mut cycle: Vec<String> = stack.iter().skip_while(|n| *n != node).cloned().collect();
            cycle.push(node.to_owned());
            return Some(cycle);
        }
        _ => {}
    }
    state.insert(node.to_owned(), ACTIVE);
    stack.push(node.to_owned());
    if let Some(targets) = refs.get(node) {
        for target in targets {
            if refs.contains_key(target.as_str())
                && let Some(cycle) = dfs(target, refs, state, stack)
            {
                return Some(cycle);
            }
        }
    }
    stack.pop();
    state.insert(node.to_owned(), DONE);
    None
}

#[cfg(test)]
#[path = "config_views_tests.rs"]
mod tests;
