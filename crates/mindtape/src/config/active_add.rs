//! [`active_add`]: every `[[on.add]]` hook a loaded config applies, with the
//! layer or pack that supplied it, for `mt explain add`.
//!
//! The typed [`Config::on`](crate::config::Config::on) already holds what the
//! merge left after same-name replacement and `disable`; the rule-origin
//! records carry the source in the same order. Zipping them is enough, so the
//! listing cannot show a hook that does not run or hide one that does.

use crate::config::Config;
use crate::config::add_hooks::AddHook;
use crate::config::pack_registry::{RuleOrigin, RuleOriginEntry};

/// One merged add hook and where it came from.
#[derive(Debug, Clone)]
pub struct ActiveAddHook<'a> {
    /// The hook as the config holds it.
    pub hook: &'a AddHook,
    /// `on.add.<name>` or `on.add[<index>]`, as `mt config show --origin`
    /// labels it.
    pub label: String,
    /// The layer or pack that supplied the hook.
    pub origin: &'a RuleOrigin,
}

/// Every `[[on.add]]` hook in merged order, paired with its origin.
///
/// `origins` is the rule-origin record a layered load returns; a config built
/// by [`crate::config::parse_config`] alone carries none, and then every hook
/// is attributed to the built-in layer. A config with no hooks returns an
/// empty list.
#[must_use]
pub fn active_add<'a>(
    config: &'a Config,
    origins: &'a [RuleOriginEntry],
) -> Vec<ActiveAddHook<'a>> {
    let mut entries = origins
        .iter()
        .filter(|entry| entry.label.starts_with("on.add"));
    config
        .on
        .add
        .iter()
        .enumerate()
        .map(|(index, hook)| {
            let entry = entries.next();
            ActiveAddHook {
                hook,
                label: entry.map_or_else(|| hook_label(hook, index), |entry| entry.label.clone()),
                origin: entry.map_or(&FALLBACK_ORIGIN, |entry| &entry.origin),
            }
        })
        .collect()
}

/// How `mt config show --origin` labels a hook with no recorded origin.
fn hook_label(hook: &AddHook, index: usize) -> String {
    hook.name.as_ref().map_or_else(
        || format!("on.add[{index}]"),
        |name| format!("on.add.{name}"),
    )
}

/// The origin used for a config with no recorded layer: the embedded floor,
/// which is where a `parse_config`-built config's hooks would live.
static FALLBACK_ORIGIN: RuleOrigin =
    RuleOrigin::Layer(crate::config::layers::LayerSource::Embedded);

#[cfg(test)]
#[path = "active_add_tests.rs"]
mod tests;
