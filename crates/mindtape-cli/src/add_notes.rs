//! The `mt add` fill note: report each config-spliced core field the template
//! did not declare and the caller did not set, naming the value, where it came
//! from, and the `--set` that overrides it. A sibling module because `add.rs`
//! sits at the linecop cap; the seam is "decide what was injected" vs "say so".

use std::collections::BTreeSet;

use anyhow::Result;
use mindtape::{RequiredSlot, discover_slots};

use crate::task_edit::task_with;

/// Print one note per config-spliced core field the caller did not set, naming
/// the value, its origin, and how to override it. Goes to stderr, so `--print`'s
/// Typst on stdout and `--format json` stay machine-clean.
pub(super) fn report_injected(
    filled: &str,
    injected: &[RequiredSlot],
    explicit: &BTreeSet<String>,
    env_keys: &BTreeSet<String>,
) -> Result<()> {
    if injected.is_empty() {
        return Ok(());
    }
    let call = task_with(filled)?;
    for slot in injected {
        if explicitly_set(slot, explicit) {
            continue;
        }
        let Some(arg) = call.named(&slot.field) else {
            continue;
        };
        let raw = &filled[arg.value_range.clone()];
        let value = display_value(raw);
        let origin = if env_keys.contains(&slot.field) {
            slot.env.map_or_else(
                || format!("{} environment", slot.config_path),
                |name| format!("{} env {name}", slot.config_path),
            )
        } else if slot.default.as_deref() == Some(raw.trim()) {
            format!("{} default", slot.config_path)
        } else {
            format!("{} spliced slot", slot.config_path)
        };
        eprintln!(
            "note: {} \"{value}\" came from {origin} — override with --set {}=<value>",
            slot.field, slot.field
        );
    }
    Ok(())
}

/// Whether the caller supplied this field itself, by `--set`/`-f`/`--`. A
/// strict-framework priority has no scalar `priority=` key; supplying any of
/// its leaves counts as taking it over, so the note stays quiet.
fn explicitly_set(slot: &RequiredSlot, explicit: &BTreeSet<String>) -> bool {
    explicit.contains(&slot.field)
        || discover_slots(&slot.call)
            .iter()
            .any(|leaf| explicit.contains(&leaf.name))
}

/// A filled argument's source, with a plain string literal's quotes stripped
/// for display. Any other literal (a number, a dict, a framework call) reads as
/// its own source.
fn display_value(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .map_or_else(|| raw.to_owned(), ToOwned::to_owned)
}
