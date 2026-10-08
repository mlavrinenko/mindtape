//! Load-time warnings: what a config stores that loads fine but is likely a
//! mistake. [`Config::validate`](crate::config::Config::validate) refuses what
//! cannot work, and this module only reports, so a command still runs.
//!
//! Today that is a stored filter naming an `x-` field `[tasks.fields]` does not
//! declare, beside fields it does: reading an undeclared name is legal, so the
//! misspelling `x-machnie` matches nothing without a word.

use crate::config::Config;
use crate::problem::Problem;

/// Every warning for `config`, each prefixed by the dotted TOML path of the
/// filter it is about.
pub(crate) fn warnings(config: &Config) -> Vec<Problem> {
    filters(config)
        .into_iter()
        .flat_map(|(path, source)| {
            crate::filter::undeclared_extras(&source, config)
                .into_iter()
                .map(move |problem| problem.under(&path))
        })
        .collect()
}

/// Every stored filter in `config` with its dotted TOML path.
fn filters(config: &Config) -> Vec<(String, String)> {
    let mut out = crate::config::required::field_filters(config);
    for (index, spec) in config.tasks.rules.iter().enumerate() {
        let path = format!("tasks.rules[{index}]");
        out.extend(
            spec.assertion
                .iter()
                .map(|expr| (format!("{path}.assert"), expr.clone())),
        );
        out.extend(
            spec.when
                .iter()
                .map(|expr| (format!("{path}.when"), expr.clone())),
        );
    }
    out.extend(
        config
            .views
            .iter()
            .map(|(name, view)| (format!("views.{name}.filter"), view.filter.clone())),
    );
    out.extend(config.on.flip.iter().enumerate().map(|(index, hook)| {
        (
            format!("on.flip[{index}].when"),
            crate::config::hooks::desugar_into(&hook.when, ""),
        )
    }));
    out
}
