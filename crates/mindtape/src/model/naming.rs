//! Task stem naming: the `[tasks.naming]` template and the shared minter.
//!
//! [`mint_stem`] is a pure macro expander: no clock reads, no filesystem
//! access. Both `mt add` (today's date, occurrence `0`) and the later `mt
//! recur` spawn path (an occurrence's anchor date, its index) build a
//! [`MintContext`] and call the same function. The two paths then can never
//! diverge. Filesystem collision handling ([`disambiguate`]) is a separate,
//! equally pure function: callers inject an `exists` predicate rather than
//! this module touching disk itself. `mint_stem` renders a template into a
//! candidate stem, and `disambiguate` decides, against the real (or faked)
//! destination directory, whether that candidate is safe to use.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::config::serialize_labels;
use crate::date::format_iso;

/// The bare-slug template, which is the behaviour when `[tasks.naming] stem`
/// is unset.
pub const DEFAULT_STEM_TEMPLATE: &str = "{slug}";

/// How `mt add` and `mt recur` name a new task's file, and what `mt add` does
/// when that name already exists (`dedup`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NamingConfig {
    /// Filename stem template. Macros: `{slug}`, `{date}`, `{date:FMT}`,
    /// `{yy}`, `{mm}`, `{dd}`, `{n}` and `{n:0N}`. Unset means `{slug}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stem: Option<String>,
    /// What `mt add` does when the stem already exists. `--dedup` overrides it
    /// for one run. Unset means `add-suffix` when `stem` has a value, else
    /// `fail`.
    // `NamingConfig::effective_dedup` holds that default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedup: Option<DedupMode>,
}

impl NamingConfig {
    /// The template to mint with, which is the configured `stem` or
    /// [`DEFAULT_STEM_TEMPLATE`] when unset.
    #[must_use]
    pub fn template(&self) -> &str {
        self.stem.as_deref().unwrap_or(DEFAULT_STEM_TEMPLATE)
    }

    /// The collision policy `mt add` falls back to when `--dedup` is absent.
    /// An explicitly configured `dedup` wins outright. Left unset, the default
    /// itself depends on `stem`. A configured stem *template* defaults to
    /// [`DedupMode::AddSuffix`], so a project that sets `stem` and never
    /// touches `dedup` keeps the behaviour it had before dedup modes reached
    /// the templated-stem path. A bare (unset) `stem` keeps
    /// [`DedupMode::Fail`], today's hard error. This is the single place the
    /// default lives, and collision *handling* itself does not branch on
    /// whether the stem came from a template (`mindtape_cli::cmd::add::dedup`).
    #[must_use]
    pub fn effective_dedup(&self) -> DedupMode {
        self.dedup.unwrap_or(if self.stem.is_some() {
            DedupMode::AddSuffix
        } else {
            DedupMode::Fail
        })
    }
}

/// `mt add`'s policy when the destination stem already exists. This is the
/// config-side counterpart of `mt add --dedup`. The CLI carries its own
/// `clap::ValueEnum` (`mindtape` has no clap dependency, see
/// `mindtape_cli::cmd::add::dedup::DedupArg`) that converts into this one, so
/// the flag and `[tasks.naming] dedup` share a single vocabulary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DedupMode {
    /// Bail before writing (today's behaviour, the default).
    #[default]
    Fail,
    /// Exit 0, leave the file unwritten, and report the existing path.
    Skip,
    /// Reuse [`disambiguate`] to append `-2`, `-3`, … until a free stem turns up.
    AddSuffix,
}

impl<'de> Deserialize<'de> for DedupMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        match text.as_str() {
            "fail" => Ok(Self::Fail),
            "skip" => Ok(Self::Skip),
            "add-suffix" => Ok(Self::AddSuffix),
            other => Err(serde::de::Error::custom(format!(
                "invalid dedup mode `{other}`; expected `fail`, `skip`, or `add-suffix`"
            ))),
        }
    }
}

serialize_labels!(DedupMode {
    Self::Fail => "fail",
    Self::Skip => "skip",
    Self::AddSuffix => "add-suffix",
});

/// Inputs to [`mint_stem`], supplied entirely by the caller: the expander
/// itself never reads a clock or a counter. `mt add` passes today's date and
/// occurrence `0`, and `mt recur` spawn passes the occurrence's anchor date
/// and its index.
#[derive(Debug, Clone, Copy)]
pub struct MintContext<'a> {
    /// The task's slug (from its title): the `{slug}` macro.
    pub slug: &'a str,
    /// Reference date `(year, month, day)`: `{date}`, `{date:FMT}`, `{yy}`,
    /// `{mm}`, `{dd}`.
    pub date: (i32, u8, u8),
    /// Occurrence index: `{n}`, `{n:0N}`.
    pub occurrence: u32,
}

/// A malformed `[tasks.naming] stem` template.
#[derive(Debug, Error)]
pub enum NamingError {
    /// A `{` was never closed with a matching `}`.
    #[error("unterminated macro (missing '}}') in stem template")]
    UnterminatedMacro,
    /// `{name}` is not a macro this expander knows.
    #[error("unknown stem template macro {{{0}}}")]
    UnknownMacro(String),
    /// `{n:ARG}`'s `ARG` is not a valid zero-pad width.
    #[error("invalid zero-pad width {0:?} in {{n:...}}")]
    InvalidWidth(String),
    /// [`disambiguate`] exhausted its numeric-suffix ceiling without finding a
    /// free stem.
    #[error("exhausted {1} disambiguation attempts for stem {0:?}")]
    Exhausted(String, u32),
}

/// Expand `template`'s macros against `ctx`. Pure: no clock, no filesystem.
///
/// Supported macros: `{slug}`, `{date}` (ISO `YYYY-MM-DD`), `{date:FMT}`
/// (formatted via [`format_iso`]'s token grammar), `{yy}` / `{mm}` / `{dd}`,
/// `{n}` (bare occurrence index) and `{n:0N}` (zero-padded to width `N`).
/// `{kind}` stays reserved for a future convergence extension (speconaut's
/// artifact-kind classifier). The expander recognises it but cannot yet
/// resolve it from [`MintContext`], so it passes through literally rather than
/// erroring or silently vanishing.
///
/// # Errors
/// Returns [`NamingError`] for an unterminated `{`, an unknown macro name, or
/// a non-numeric `{n:...}` width.
pub fn mint_stem(template: &str, ctx: &MintContext<'_>) -> Result<String, NamingError> {
    let mut out = String::with_capacity(template.len());
    let mut chars = template.chars();
    while let Some(ch) = chars.next() {
        if ch != '{' {
            out.push(ch);
            continue;
        }
        let inner = read_macro(&mut chars)?;
        out.push_str(&expand_macro(&inner, ctx)?);
    }
    Ok(out)
}

/// Consume `chars` up to (and including) the closing `}`, returning the
/// content between them. Errors if the input ends first.
fn read_macro(chars: &mut std::str::Chars<'_>) -> Result<String, NamingError> {
    let mut inner = String::new();
    for ch in chars {
        if ch == '}' {
            return Ok(inner);
        }
        inner.push(ch);
    }
    Err(NamingError::UnterminatedMacro)
}

fn expand_macro(inner: &str, ctx: &MintContext<'_>) -> Result<String, NamingError> {
    let (name, arg) = inner
        .split_once(':')
        .map_or((inner, None), |(n, a)| (n, Some(a)));
    let (year, month, day) = ctx.date;
    let iso = format!("{year:04}-{month:02}-{day:02}");
    match name {
        "slug" => Ok(ctx.slug.to_owned()),
        "date" => Ok(match arg {
            None => iso,
            Some(fmt) => format_iso(&iso, fmt),
        }),
        "yy" => Ok(format_iso(&iso, "YY")),
        "mm" => Ok(format_iso(&iso, "MM")),
        "dd" => Ok(format_iso(&iso, "DD")),
        "n" => expand_occurrence(ctx.occurrence, arg),
        // Reserved convergence hook: recognised, not yet resolvable from
        // `MintContext` — pass through literally so an authored `{kind}` is
        // visibly unresolved rather than silently dropped.
        "kind" => Ok(format!("{{{inner}}}")),
        other => Err(NamingError::UnknownMacro(other.to_owned())),
    }
}

fn expand_occurrence(occurrence: u32, arg: Option<&str>) -> Result<String, NamingError> {
    let Some(width_arg) = arg else {
        return Ok(occurrence.to_string());
    };
    let width: usize = width_arg
        .parse()
        .map_err(|_| NamingError::InvalidWidth(width_arg.to_owned()))?;
    Ok(format!("{occurrence:0width$}"))
}

/// Ceiling on numeric suffixes [`disambiguate`] will try before giving up.
/// `10_000` covers a daily recurring task disambiguating against a full
/// destination directory for roughly 27 years, comfortably past any realistic
/// collision run. It still bounds the search against a pathological or buggy
/// `exists` predicate.
const MAX_DISAMBIGUATE_ATTEMPTS: u32 = 10_000;

/// Disambiguate `stem` against an `exists` predicate: appends an incrementing
/// `-{n}` (`stem`, `stem-2`, `stem-3`, …) until a candidate reports as free.
/// Never returns a stem `exists` reports as taken. Fs-free: callers pass a
/// closure over the real filesystem (the `add` destination dir) or a fake set
/// in tests.
///
/// # Errors
/// Returns [`NamingError::Exhausted`] after [`MAX_DISAMBIGUATE_ATTEMPTS`]
/// numeric suffixes are all reported taken, rather than looping forever or
/// returning a stem `exists` reports as taken.
pub fn disambiguate(stem: &str, exists: impl Fn(&str) -> bool) -> Result<String, NamingError> {
    if !exists(stem) {
        return Ok(stem.to_owned());
    }
    // A `for` over a fixed range, not a hand-incremented counter: the bound
    // then holds however the loop body behaves.
    for n in 2..=MAX_DISAMBIGUATE_ATTEMPTS {
        let candidate = format!("{stem}-{n}");
        if !exists(&candidate) {
            return Ok(candidate);
        }
    }
    Err(NamingError::Exhausted(
        stem.to_owned(),
        MAX_DISAMBIGUATE_ATTEMPTS,
    ))
}

#[cfg(test)]
#[path = "naming_tests.rs"]
mod tests;
