//! `Config::validate`: the semantic checks that go beyond TOML shape, run
//! over a parsed config and every layer merged into it.

use crate::config::Config;
use crate::config::dicts::validate_dicts;
use crate::config::ls::validate_ls_rules_with;
use crate::config::ls_columns::validate_ls_columns;
use crate::config::views::{validate_view_statuses, validate_views_with};
use crate::problem::Problem;
use crate::suggest::Unknown;

/// String problems as [`Problem`]s with nothing to suggest.
fn plain(problems: Vec<String>) -> impl Iterator<Item = Problem> {
    problems.into_iter().map(Problem::from)
}

impl Config {
    /// Run semantic checks that go beyond TOML shape (numeric scale and
    /// classification coverage). Returns the problems joined one per line.
    ///
    /// # Errors
    /// Returns the collected problems when the numeric fields are inconsistent.
    pub fn validate(&self) -> Result<(), Problem> {
        let mut problems: Vec<Problem> = plain(self.tasks.priority.validate("priority")).collect();
        problems.extend(plain(self.tasks.difficulty.validate("difficulty")));
        problems.extend(self.tasks.status.transition_problems());
        problems.extend(plain(crate::loader::derive::validate_root_globs(self)));
        for spec in &self.ls.default_sort {
            if let Err(msg) = crate::sort::validate_sort_spec_with(spec, &self.tasks.collectors) {
                problems.push(msg.under("ls.default-sort"));
            }
        }
        problems.extend(validate_ls_columns(&self.ls));
        problems.extend(validate_views_with(&self.views, &self.tasks.collectors));
        problems.extend(validate_view_statuses(&self.views, &self.tasks.status));
        problems.extend(validate_dicts(&self.dicts));
        problems.extend(plain(crate::config::tasks::validate_field_names(
            &self.tasks,
        )));
        problems.extend(crate::config::collect::validate_collectors(self));
        problems.extend(unknown_derived_refs(self));
        problems.extend(crate::config::required::validate_requirements(self));
        problems.extend(crate::config::required::validate_values_when(self));
        problems.extend(plain(crate::config::links_validate::validate_link_kinds(
            self,
        )));
        problems.extend(crate::rules::validate_rules(self));
        problems.extend(plain(crate::rules::validate_rule_names(self)));
        problems.extend(crate::config::commits::validate_commits(self));
        problems.extend(plain(crate::body::headers::validate_header_rules(self)));
        problems.extend(plain(crate::body::content::validate_content_rules(self)));
        problems.extend(plain(crate::config::comments::validate_comments(self)));
        problems.extend(crate::config::actors::validate_actors(self));
        problems.extend(crate::config::hooks::validate_flip_hooks(self));
        // Two entries sharing a name can only come from one file: the merge
        // replaced any pair from different sources.
        problems.extend(plain(crate::body::headers::duplicate_rule_names(
            self.on.flip.iter().map(|hook| hook.name.as_deref()),
            "on.flip",
        )));
        problems.extend(plain(crate::body::headers::duplicate_rule_names(
            self.on.add.iter().map(|hook| hook.name.as_deref()),
            "on.add",
        )));
        problems.extend(plain(crate::waits::validate_checkers(self)));
        problems.extend(crate::estimate_queries::validate_config(self));
        problems.extend(validate_ls_rules_with(
            &self.ls.rules,
            &self.views,
            &self.tasks.collectors,
        ));
        if let Some(status) = &self.retire.status
            && !self.tasks.status.contains(status)
        {
            problems.push(format!("retire.status: unknown status `{status}`").into());
        }
        if let Some(name) = &self.ls.default_view
            && !self.views.contains_key(name)
        {
            problems.push(
                Problem::new(format!("ls.default-view: unknown view `{name}`"))
                    .with(Unknown::among(name, self.views.keys().map(String::as_str))),
            );
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(Problem::join(problems))
        }
    }
}

/// The first undeclared `derived.<name>` each saved task filter reads, one
/// problem per expression. The dry runs cover `[[tasks.rules]]`, `required`
/// and `[[on.flip]]`; these are the filters only parsed elsewhere.
fn unknown_derived_refs(config: &Config) -> Vec<Problem> {
    let mut problems = Vec::new();
    let mut check = |at: String, source: &str| {
        if let Some(problem) = crate::filter::unknown_derived(source, config) {
            problems.push(problem.under(&at));
        }
    };
    for (index, rule) in config.tasks.body.headers.rules.iter().enumerate() {
        if let Some(when) = &rule.when {
            check(format!("tasks.body.headers.rules[{index}].when"), when);
        }
    }
    for (index, rule) in config.tasks.body.content.rules.iter().enumerate() {
        if let Some(when) = &rule.when {
            check(format!("tasks.body.content.rules[{index}].when"), when);
        }
    }
    for (name, view) in &config.views {
        check(format!("views.{name}.filter"), &view.filter);
    }
    for (index, rule) in config.ls.rules.iter().enumerate() {
        if let Some(filter) = &rule.filter {
            check(format!("ls.rules[{index}].filter"), filter);
        }
    }
    if let Some(filter) = &config.rm.default_filter {
        check("rm.default-filter".to_owned(), filter);
    }
    problems
}
