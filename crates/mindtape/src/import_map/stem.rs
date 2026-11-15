//! Deterministic stem minting from a title: `slug_from_title` is the one
//! slug rule, re-exported for `mindtape_cli::cmd::add::values::slug` to call so
//! `mt import` and `mt add` mint identical stems from an identical title.
//! The CLI owns collision handling ([`crate::model::naming::disambiguate`] against
//! the real destination directory). This module only computes the deterministic
//! candidate.

use crate::config::Config;
use crate::model::naming::{MintContext, NamingError, mint_stem};

/// Slugify `title` into a filename-stem-safe token. Lowercase Unicode
/// alphanumerics survive (Cyrillic, wide scripts, accented Latin all kept).
/// The function drops apostrophes (`'` and `’`) rather than treating them as
/// separators, collapses other runs to a single `-`, and trims leading and
/// trailing dashes from the result.
#[must_use]
pub fn slug_from_title(title: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for ch in title.chars() {
        if ch == '\'' || ch == '’' {
            continue;
        }
        if ch.is_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.extend(ch.to_lowercase());
        } else {
            pending_dash = true;
        }
    }
    out
}

/// The deterministic candidate stem for `title` under `config`'s
/// `[tasks.naming]` template, given a reference `date` (the CLI passes
/// today's date, and the pure expander never reads a clock). Returns the bare
/// slug unchanged when the config leaves `stem` unset. Never disambiguates
/// against an existing file. That is the CLI's job (see the module doc).
///
/// # Errors
/// Returns [`NamingError`] when the configured `[tasks.naming] stem` template
/// is invalid.
pub fn stem_for_title(
    title: &str,
    config: &Config,
    date: (i32, u8, u8),
) -> Result<String, NamingError> {
    let slug = slug_from_title(title);
    let Some(template) = config.tasks.naming.stem.as_deref() else {
        return Ok(slug);
    };
    let ctx = MintContext {
        slug: &slug,
        date,
        occurrence: 0,
    };
    mint_stem(template, &ctx)
}
