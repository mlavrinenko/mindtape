//! Where each `[[tasks.rules]]` entry is declared, so a report that is about
//! the rule rather than a task opens the rule.
//!
//! [`crate::config::packs`] already records which layer or pack each merged rule
//! came from, for `mt config show --origin`, and the line of its table header.
//! [`crate::config::layers::build_config`] turns that record into one
//! [`Location`] per rule and keeps it on the loaded [`crate::config::Config`],
//! because the validation passes that report a skipped rule take a `Config`
//! and nothing else. A rule from the project config, a global layer, a path
//! pack or a built-in pack all resolve the same way.

use std::collections::BTreeMap;
use std::path::Path;

use crate::config::layers::LayerSource;
use crate::config::packs::{PackOrigin, RuleOrigin, RuleOriginEntry};
use crate::violation::Location;

/// The label prefix of the `[[tasks.rules]]` entries in a rule origin record.
const RULES_PREFIX: &str = "tasks.";

/// Each `[[tasks.rules]]` entry's location, by the label its reports carry
/// (`rules.<name>` or `rules[<index>]`).
///
/// Derived at load, never declared, never equal-compared: two configs with the
/// same declared content are the same config wherever their rules came from,
/// the reading [`crate::rules::RulesRoot`] takes for the same kind of
/// load-derived state. Empty for a [`crate::config::Config`] built through
/// [`crate::config::parse_config`] alone.
#[derive(Debug, Clone, Default)]
pub(crate) struct RuleSites(BTreeMap<String, Location>);

impl PartialEq for RuleSites {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl RuleSites {
    /// The sites of every `tasks.rules` entry in `origins`, files shown
    /// relative to `root` when they sit under it.
    pub(crate) fn from_origins(origins: &[RuleOriginEntry], root: &Path) -> Self {
        let sites = origins
            .iter()
            .filter(|entry| entry.label.starts_with("tasks.rules"))
            .filter_map(|entry| {
                let label = entry.label.strip_prefix(RULES_PREFIX)?;
                Some((label.to_owned(), site(entry, root)?))
            })
            .collect();
        Self(sites)
    }

    /// Where the rule reported as `label` is declared, if the load recorded it.
    pub(crate) fn get(&self, label: &str) -> Option<&Location> {
        self.0.get(label)
    }
}

/// One entry's location: its source file, or the built-in pack's name.
fn site(entry: &RuleOriginEntry, root: &Path) -> Option<Location> {
    let line = entry.line?;
    match &entry.origin {
        RuleOrigin::Pack(PackOrigin::Embedded(name)) => Some(Location::Pack {
            pack: name.clone(),
            line,
        }),
        RuleOrigin::Pack(PackOrigin::Path(path))
        | RuleOrigin::Layer(
            LayerSource::Xdg(path) | LayerSource::Dir(path) | LayerSource::Project(path),
        ) => Some(Location::at(relative(path, root), line)),
        RuleOrigin::Layer(LayerSource::Embedded) => None,
    }
}

fn relative(path: &Path, root: &Path) -> std::path::PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

#[cfg(test)]
#[path = "sites_tests.rs"]
mod tests;
