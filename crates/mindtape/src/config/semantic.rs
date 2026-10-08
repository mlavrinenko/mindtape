//! `Config::validate`: the semantic checks that go beyond TOML shape, run
//! over a parsed config and every layer merged into it.

use crate::config::Config;
use crate::config::dicts::validate_dicts;
use crate::config::ls::validate_ls_rules;
use crate::config::ls_columns::validate_ls_columns;
use crate::config::views::{validate_view_statuses, validate_views};
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
            if let Err(msg) = crate::sort::validate_sort_spec(spec) {
                problems.push(msg.under("ls.default-sort"));
            }
        }
        problems.extend(validate_ls_columns(&self.ls));
        problems.extend(validate_views(&self.views));
        problems.extend(plain(validate_view_statuses(
            &self.views,
            &self.tasks.status,
        )));
        problems.extend(validate_dicts(&self.dicts));
        problems.extend(plain(crate::config::tasks::validate_field_names(
            &self.tasks,
        )));
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
        problems.extend(validate_ls_rules(&self.ls.rules, &self.views));
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
