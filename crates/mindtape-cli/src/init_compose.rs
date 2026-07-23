//! Fold the `mt init` wizard's axis fragments into one config body.
//!
//! Each configuration axis the wizard walks answers with one TOML fragment
//! ([`crate::init_wizard`]); this module is the single place those fragments
//! become one `.mindtape/config.toml`. Keeping the merge in one small function
//! is deliberate: the sibling task `preset-overlay-and-compose` extends exactly
//! this spot — appending an `[overlay]` fragment last and promoting an
//! array-valued collision to a hard error — without reaching into the wizard.

use anyhow::{Context, Result};

/// Fold ordered axis fragments into one flattened config body.
///
/// Each fragment is a self-contained, readable TOML snippet that owns disjoint
/// sections: it uses only sub-table (`[tasks.priority]`) and array-of-table
/// (`[[tasks.status]]`) headers — never a bare `[tasks]` — so no two fragments
/// re-open the same table. The fold concatenates the non-empty fragments in the
/// order given, preserving each one's own comments and layout, then re-parses
/// the whole as one TOML document so a collision (a table two fragments both
/// define) surfaces here rather than downstream at the config loader.
///
/// A fragment is `(axis name, TOML text)`; the axis name rides along for error
/// context and for the sibling overlay extension, which appends its
/// `("overlay", …)` fragment last and rejects an array-valued conflict. Today
/// the axes are disjoint, so this plain ordered fold is enough.
///
/// # Errors
/// Returns an error when the concatenation is not one valid TOML document — the
/// signal that two fragments defined the same table.
pub fn compose(fragments: &[(&str, String)]) -> Result<String> {
    let mut out = String::new();
    for (_axis, text) in fragments {
        let body = text.trim();
        if body.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(body);
        out.push('\n');
    }
    toml::from_str::<toml::Table>(&out).context(
        "composed axis fragments do not form one valid TOML document \
         (two fragments may define the same table)",
    )?;
    Ok(out)
}

#[cfg(test)]
#[path = "init_compose_tests.rs"]
mod tests;
