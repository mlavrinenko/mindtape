//! Which pack or layer set each key and each pack parameter, for `mt config
//! show --origin`. The merge feeds [`Sources`], the carrier a load error and a
//! load warning already ask for a file and line, with each source's leaf keys
//! in precedence order ([`Sources::record_keys`]) and each pack's resolved
//! parameters ([`Sources::record_params`]). [`Config`](crate::config::Config)
//! keeps it, so a caller reads both without a second load.
//!
//! A key is listed only when some pack set it. Its origin is the source that
//! wins, so a key the extending layer replaced names the layer.

use crate::config::layers::LayerSource;
use crate::config::locate::Sources;
use crate::config::pack_registry::{PackOrigin, RuleOrigin};
use crate::config::params::ResolvedParam;

/// The source that set one leaf key of the merged config, which a pack also
/// set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyOrigin {
    /// The dotted path, such as `tasks.priority.framework`.
    pub key: String,
    /// The layer or pack whose value won.
    pub origin: RuleOrigin,
}

/// One parameter of one pack, resolved for an `extends` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamOrigin {
    /// The parameter's name in the pack's `[params]`.
    pub name: String,
    /// The value, as TOML writes it.
    pub value: String,
    /// The pack that declares the parameter.
    pub pack: PackOrigin,
    /// The layer whose `with` set it, or `None` for the pack's default.
    pub set_by: Option<LayerSource>,
}

/// A leaf key's winning source so far, and whether a pack ever set it.
#[derive(Debug)]
pub(super) struct KeySlot {
    origin: RuleOrigin,
    by_pack: bool,
}

impl Sources {
    /// Note every leaf of `table`, a source's own table with its rule arrays
    /// and `[params]` already taken out. Call it for each source in merge
    /// precedence, lowest first, so a later source wins.
    pub(crate) fn record_keys(&mut self, origin: &RuleOrigin, table: &toml::Table) {
        let by_pack = matches!(origin, RuleOrigin::Pack(_));
        record_leaves(self, origin, by_pack, "", table);
    }

    /// Note each of a pack's resolved parameters. `layer` is the layer whose
    /// `extends` entry named the pack, so it is the one a `with` value came
    /// from.
    pub(crate) fn record_params(
        &mut self,
        layer: &LayerSource,
        pack: &PackOrigin,
        params: &[ResolvedParam],
    ) {
        self.params.extend(params.iter().map(|param| ParamOrigin {
            name: param.name.clone(),
            value: param.value.to_string(),
            pack: pack.clone(),
            set_by: param.from_with.then(|| layer.clone()),
        }));
    }

    /// The keys a pack set, each with the source that won it, by path.
    pub(crate) fn key_origins(&self) -> impl Iterator<Item = KeyOrigin> + '_ {
        self.keys
            .iter()
            .filter(|(_, slot)| slot.by_pack)
            .map(|(key, slot)| KeyOrigin {
                key: key.clone(),
                origin: slot.origin.clone(),
            })
    }

    /// Every pack parameter, in the order the layers named their packs.
    pub(crate) fn param_origins(&self) -> &[ParamOrigin] {
        &self.params
    }
}

/// A table recurses and anything else, an array included, is a leaf, as a
/// merge replaces an array whole.
fn record_leaves(
    sources: &mut Sources,
    origin: &RuleOrigin,
    by_pack: bool,
    prefix: &str,
    table: &toml::Table,
) {
    for (name, value) in table {
        let key = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        if let toml::Value::Table(inner) = value {
            record_leaves(sources, origin, by_pack, &key, inner);
            continue;
        }
        let slot = sources.keys.entry(key).or_insert_with(|| KeySlot {
            origin: origin.clone(),
            by_pack,
        });
        slot.origin = origin.clone();
        slot.by_pack |= by_pack;
    }
}
