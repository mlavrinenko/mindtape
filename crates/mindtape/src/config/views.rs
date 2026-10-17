//! `[views]` config: named query bundles.
//!
//! A view names a filter plus optional sort and column set. It is referenced
//! from the CLI by an `@name` sigil and from the filter language by
//! `in_view("name")`. Validated at config load ([`validate_views`]): each filter
//! parses, sort specs and columns are legal, every `in_view` / `@` reference
//! resolves to a declared view, and the reference graph is acyclic.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::ls::LS_COLUMNS;
use crate::config::ls_columns::unknown_column;
use crate::config::name_cycle::find_cycle;
use crate::filter::Filter;
use crate::problem::Problem;
use crate::sort::validate_sort_spec;
use crate::suggest::Unknown;

/// One named query bundle.
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
pub fn validate_views(views: &BTreeMap<String, ViewSpec>) -> Vec<Problem> {
    let mut problems = Vec::new();
    let mut refs: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for (name, spec) in views {
        match Filter::parse(&spec.filter) {
            Ok(filter) => {
                let named = filter.view_refs();
                for target in &named {
                    if !views.contains_key(target) {
                        problems.push(
                            Problem::new(format!(
                                "views.{name}: unknown view: {}",
                                super::quote(target)
                            ))
                            .with(Unknown::near(
                                target,
                                &views.keys().map(String::as_str).collect::<Vec<_>>(),
                            )),
                        );
                    }
                }
                refs.insert(name.as_str(), named);
            }
            Err(err) => problems.push(format!("views.{name}.filter: {err}").into()),
        }
        for spec_str in &spec.sort {
            if let Err(msg) = validate_sort_spec(spec_str) {
                problems.push(msg.under(&format!("views.{name}.sort")));
            }
        }
        for col in &spec.columns {
            if !LS_COLUMNS.contains(&col.as_str()) {
                problems.push(unknown_column(col).under(&format!("views.{name}.columns")));
            }
        }
    }
    if let Some(cycle) = find_cycle(&refs) {
        problems.push(format!("views: reference cycle: {}", cycle.join(" -> ")).into());
    }
    problems
}

/// Refuse a view whose filter compares `status` with a value the legend does
/// not declare, one message per view. A filter that does not parse is
/// [`validate_views`]'s to report.
#[must_use]
pub fn validate_view_statuses(
    views: &BTreeMap<String, ViewSpec>,
    legend: &crate::config::Legend,
) -> Vec<Problem> {
    views
        .iter()
        .filter_map(|(name, spec)| {
            let err = Filter::parse(&spec.filter)
                .ok()?
                .check_statuses(legend)
                .err()?;
            Some(crate::filter::problem_of(&err).under(&format!("views.{name}.filter")))
        })
        .collect()
}

#[cfg(test)]
#[path = "views_tests.rs"]
mod tests;
