//! Shared TOML deep-merge: a base document overlaid by another, field-by-field.
//!
//! Used by both the built-in priority-frameworks merge
//! ([`crate::framework::merge_builtin_frameworks`]) and the embedded config
//! floor ([`crate::config::parse_config`]).

/// Merge `overlay` into `base` in place: tables merge key-by-key (recursing);
/// arrays and scalars replace the base value wholesale.
pub(crate) fn deep_merge(base: &mut toml::Value, overlay: toml::Value) {
    match overlay {
        toml::Value::Table(over) => {
            let toml::Value::Table(base) = base else {
                *base = toml::Value::Table(over);
                return;
            };
            deep_merge_tables(base, over);
        }
        other => *base = other,
    }
}

/// Merge `overlay` into `base` in place, both already known to be tables (the
/// common top-level case: two parsed documents). Shares the recursive
/// value-level merge with [`deep_merge`], whose `Table` arm delegates here.
pub(crate) fn deep_merge_tables(base: &mut toml::Table, overlay: toml::Table) {
    for (key, value) in overlay {
        match base.get_mut(&key) {
            Some(existing) => deep_merge(existing, value),
            None => {
                base.insert(key, value);
            }
        }
    }
}

#[cfg(test)]
#[path = "merge_tests.rs"]
mod tests;
