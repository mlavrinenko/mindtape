//! One layer's packs folded into the sub-layer that merges just below it.
//!
//! [`crate::config::packs`] resolves a layer's `extends` entries in order,
//! takes each pack's rule arrays out for their merge by name, and hands the
//! rest of the pack to [`PackLayer::add`]. That checks it against every
//! earlier pack of the same layer, then deep-merges it in with
//! [`deep_merge_tables`]. A leaf value, a scalar or an array, that two packs
//! set differently is an error naming the dotted key and both packs, unless
//! the extending layer sets that key itself and so picks the value. Equal
//! values agree.
//!
//! The finished sub-layer merges into the document after the earlier layers
//! and before the extending layer's own keys. So the layer beats its packs,
//! its packs beat every earlier layer, and the embedded floor shows through
//! wherever nothing above it sets a key. Packs of two different layers never
//! conflict: the later layer's wins, as for any two layers.
//!
//! [`PackLayer`] keeps each pack's own table and resolved parameters beside the
//! merge, in `extends` order, so [`crate::config::origins`] can name the pack
//! behind each value.

use std::path::Path;

use crate::config::ConfigError;
use crate::config::merge::deep_merge_tables;
use crate::config::pack_registry::PackOrigin;
use crate::config::params::ResolvedParam;

/// The packs of one layer's `extends`, folded so far.
#[derive(Default)]
pub(super) struct PackLayer {
    /// Every pack added so far, deep-merged.
    merged: toml::Table,
    /// Each added pack and its table, rule arrays already taken out.
    packs: Vec<(PackOrigin, toml::Table)>,
    /// Each added pack's source text, for the file an error names.
    texts: Vec<(PackOrigin, String)>,
    /// Each added pack's resolved parameters.
    params: Vec<(PackOrigin, Vec<ResolvedParam>)>,
}

impl PackLayer {
    /// Fold one pack in, after every earlier pack of this layer. `layer` is
    /// the extending layer's own table: a key it sets settles any conflict
    /// between its packs, since its value wins over both.
    ///
    /// # Errors
    /// Returns [`ConfigError::Invalid`] when this layer already added the
    /// pack, or the pack sets a value an earlier pack of this layer set
    /// differently and `layer` doesn't set it.
    pub(super) fn add(
        &mut self,
        origin: PackOrigin,
        table: toml::Table,
        layer: &toml::Table,
    ) -> Result<(), ConfigError> {
        if self.packs.iter().any(|(earlier, _)| *earlier == origin) {
            return Err(ConfigError::invalid(format!(
                "extends: pack {} is listed twice",
                crate::config::quote(&origin.label())
            )));
        }
        for (earlier, earlier_table) in &self.packs {
            if let Some(key) = first_conflict(earlier_table, &table, Some(layer)) {
                return Err(ConfigError::invalid(format!(
                    "extends: packs {} and {} set different values for: {}",
                    crate::config::quote(&earlier.label()),
                    crate::config::quote(&origin.label()),
                    crate::config::quote(&key)
                ))
                .with_hint("set the key in the config that extends both packs to pick one"));
            }
        }
        deep_merge_tables(&mut self.merged, table.clone());
        self.packs.push((origin, table));
        Ok(())
    }

    /// Keep a pack's source text beside its table.
    pub(super) fn keep_text(&mut self, origin: PackOrigin, text: String) {
        self.texts.push((origin, text));
    }

    /// Keep a pack's resolved parameters beside its table.
    pub(super) fn keep_params(&mut self, origin: PackOrigin, params: Vec<ResolvedParam>) {
        self.params.push((origin, params));
    }

    /// Each added pack's resolved parameters, in `extends` order.
    pub(super) fn params(&self) -> &[(PackOrigin, Vec<ResolvedParam>)] {
        &self.params
    }

    /// Each added pack's own table, rule arrays and `[params]` taken out, in
    /// `extends` order.
    pub(super) fn tables(&self) -> &[(PackOrigin, toml::Table)] {
        &self.packs
    }

    /// Each added pack's origin and source text, in `extends` order.
    pub(super) fn texts(&self) -> &[(PackOrigin, String)] {
        &self.texts
    }

    /// The directory of each directory pack added so far, in `extends` order.
    pub(super) fn dirs(&self) -> impl Iterator<Item = &Path> {
        self.packs.iter().filter_map(|(origin, _)| origin.dir())
    }

    /// The sub-layer to merge below the layer that extends these packs.
    pub(super) fn into_table(self) -> toml::Table {
        self.merged
    }
}

/// The dotted key of the first value `a` and `b` both set, differently,
/// that `layer` leaves open. Tables recurse key by key; any other pair, a
/// table against a scalar included, conflicts unless equal. `layer` settles
/// a key it sets, and a non-table it sets settles everything under it.
fn first_conflict(a: &toml::Table, b: &toml::Table, layer: Option<&toml::Table>) -> Option<String> {
    b.iter().find_map(|(key, b_value)| {
        let a_value = a.get(key)?;
        let settled = layer.and_then(|layer| layer.get(key));
        match (a_value, b_value) {
            (toml::Value::Table(a), toml::Value::Table(b)) => {
                let layer = match settled {
                    Some(toml::Value::Table(layer)) => Some(layer),
                    Some(_) => return None,
                    None => None,
                };
                first_conflict(a, b, layer).map(|rest| format!("{key}.{rest}"))
            }
            _ => (a_value != b_value && settled.is_none()).then(|| key.clone()),
        }
    })
}
