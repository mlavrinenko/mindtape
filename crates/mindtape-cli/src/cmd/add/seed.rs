//! The caller's values before the editor: `-f`/`--set`/`--` operations, the
//! scalar flags and the environment, collected into the fill map `mt add`
//! starts from.
//!
//! The module split out of `pipeline.rs` to keep that file under its line
//! budget. The render, the hooks and the interactive loop stay there.

use std::collections::{BTreeMap, BTreeSet};

use mindtape::model::writable::FieldOp;
use mindtape::{RawValue, SlotSpec};

use crate::diag::{Diag, Hint, Result};

use super::title::{fill_title_from_args_in, fill_title_from_stdin_in, split_positional};
use super::values::{Collected, collect_values, layer_env, reject_unknown_keys};

/// The caller's values before the editor: the ordered operations, the fill
/// map, the fields they set, and the keys the environment added.
pub(super) struct Seeded {
    pub(super) collected: Collected,
    pub(super) values: BTreeMap<String, RawValue>,
    pub(super) explicit: BTreeSet<String>,
    pub(super) env_keys: BTreeSet<String>,
}

/// Collect the caller's `-f`/`--set`/`--` values, the scalar flags and the
/// environment into the fill map the editor starts from.
pub(super) fn seed(
    args: &super::AddArgs,
    specs: &[SlotSpec],
    aliases: &BTreeMap<String, String>,
    title_slot: &str,
) -> Result<Seeded> {
    let (title_words, tags) = split_positional(args, specs);
    let mut collected = collect_values(&args.values, &args.set, &args.tokens)?;
    let tag_ops = tags
        .into_iter()
        .map(|tag| ("tags".to_owned(), FieldOp::Append(tag)));
    collected.ops.splice(0..0, tag_ops);
    let mut flags = flag_ops(args);
    refuse_flag_and_set(args, &flags, aliases)?;
    route_ops(&mut collected.ops, aliases);
    route_ops(&mut flags, aliases);
    collected.ops.extend(flags);
    let mut explicit = super::explicit_sets(&collected.ops);
    reject_unknown_keys(&collected.ops, specs)?;
    let mut values = collected.slot_values(specs)?;
    if let Some(slot) = fill_title_from_args_in(&mut values, &title_words, title_slot) {
        explicit.insert("title".to_owned());
        explicit.insert(slot);
    }
    let before_env: BTreeSet<String> = values.keys().cloned().collect();
    layer_env(&mut values, specs, |key| std::env::var(key).ok());
    let env_keys: BTreeSet<String> = values
        .keys()
        .filter(|key| !before_env.contains(*key))
        .cloned()
        .collect();
    if let Some(slot) = fill_title_from_stdin_in(&mut values, args, title_slot)? {
        explicit.insert("title".to_owned());
        explicit.insert(slot);
    }
    Ok(Seeded {
        collected,
        values,
        explicit,
        env_keys,
    })
}

/// Every scalar `mt add` flag as an ordered field write, in the same shape
/// `--set field=value` builds.
fn flag_ops(args: &super::AddArgs) -> Vec<(String, FieldOp)> {
    [
        ("priority", &args.priority),
        ("difficulty", &args.difficulty),
        ("kind", &args.kind),
        ("assignee", &args.assignee),
        ("due", &args.due),
        ("start", &args.start),
    ]
    .into_iter()
    .filter_map(|(name, value)| {
        value
            .as_ref()
            .map(|value| (name.to_owned(), FieldOp::Set(RawValue::Str(value.clone()))))
    })
    .collect()
}

/// Refuse a scalar flag and a `--set`/after-`--` token naming the same field:
/// two spellings of one value is a caller mistake, not a precedence rule. A
/// slot name and its core field (`type` and `kind`) are one field here.
fn refuse_flag_and_set(
    args: &super::AddArgs,
    flags: &[(String, FieldOp)],
    aliases: &BTreeMap<String, String>,
) -> Result<()> {
    for (name, _) in flags {
        let repeated = args.set.iter().chain(&args.tokens).any(|token| {
            super::tokens::token_name(token).is_ok_and(|token| {
                canonical_field(token, aliases) == canonical_field(name, aliases)
            })
        });
        if repeated {
            return Err(Diag::usage("the same field is set twice")
                .value(name.clone())
                .hint(Hint::new("pass the value once, as a flag or as --set")));
        }
    }
    Ok(())
}

/// The core field that a name addresses. A slot alias maps back to the field it
/// fills, and a core name maps to itself.
fn canonical_field<'a>(name: &'a str, aliases: &'a BTreeMap<String, String>) -> &'a str {
    aliases.get(name).map_or(name, String::as_str)
}

/// Rewrite a core field name to the slot that fills it. `--kind feat` against
/// `kind: slot("type")` becomes a write of `type`. The slot then keeps its
/// value, and the field writer does not replace the whole argument.
pub(super) fn route_ops(ops: &mut [(String, FieldOp)], aliases: &BTreeMap<String, String>) {
    let by_field: BTreeMap<&str, &str> = aliases
        .iter()
        .map(|(slot, field)| (field.as_str(), slot.as_str()))
        .collect();
    for (name, _) in ops {
        if let Some(slot) = by_field.get(name.as_str()) {
            *name = (*slot).to_owned();
        }
    }
}
