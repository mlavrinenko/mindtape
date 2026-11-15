//! `dir` template parsing and validation, split out of `add_hooks.rs` for the
//! file-size gate.
//!
//! A `dir` is project-root-relative text with `{macro}` placeholders. A static
//! template may reference `{template}`, any add-safe scalar field, and the date
//! macros `{date}`, `{date:FMT}`, `{yy}`, `{mm}` and `{dd}`, all in the
//! `[tasks.naming] stem` grammar. A value a derive function returns is a plain
//! path and skips macro expansion, but must still stay relative and free of
//! `.`/`..`.

use std::path::Path;

use crate::config::add_hooks::is_add_safe;
use crate::date::format_iso;

/// The one non-field macro a `dir` template takes.
pub(super) const TEMPLATE_MACRO: &str = "template";

/// The date a [`is_date_macro`] probe formats with. The probe checks the name's
/// shape only, never the value.
const PROBE_DATE: (i32, u8, u8) = (2000, 1, 1);

/// One date macro's value for `date`: `{date}` in ISO, `{date:FMT}` through
/// [`format_iso`]'s token grammar, and `{yy}`, `{mm}`, `{dd}` as `YY`, `MM`,
/// `DD`. `None` for any other name, so the accepted names and the expansion
/// share one match.
#[must_use]
pub fn date_macro_value(name: &str, date: (i32, u8, u8)) -> Option<String> {
    let (year, month, day) = date;
    let iso = format!("{year:04}-{month:02}-{day:02}");
    match name {
        "date" => Some(iso),
        "yy" => Some(format_iso(&iso, "YY")),
        "mm" => Some(format_iso(&iso, "MM")),
        "dd" => Some(format_iso(&iso, "DD")),
        _ => name.strip_prefix("date:").map(|fmt| format_iso(&iso, fmt)),
    }
}

/// Whether `name` is one of the date macros a `dir` template takes, in the same
/// grammar as `[tasks.naming] stem`.
#[must_use]
pub fn is_date_macro(name: &str) -> bool {
    date_macro_value(name, PROBE_DATE).is_some()
}

/// One piece of a `dir` template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirSegment<'a> {
    /// Text copied through verbatim, slashes included.
    Literal(&'a str),
    /// A `{name}` macro, expanded by [`DirTemplate::expand`].
    Macro(&'a str),
}

/// A parsed `dir` template: literal path text and `{macro}` placeholders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirTemplate<'a> {
    segments: Vec<DirSegment<'a>>,
}

impl<'a> DirTemplate<'a> {
    /// Parse `text` into literal and macro segments.
    ///
    /// # Errors
    /// Returns a message for an unclosed `{` or an empty `{}` macro.
    pub fn parse(text: &'a str) -> Result<Self, String> {
        let mut segments = Vec::new();
        let mut rest = text;
        while let Some(open) = rest.find('{') {
            let (literal, after) = rest.split_at(open);
            if !literal.is_empty() {
                segments.push(DirSegment::Literal(literal));
            }
            let Some((name, tail)) = after
                .strip_prefix('{')
                .and_then(|body| body.split_once('}'))
            else {
                return Err("unclosed `{` in dir template".to_owned());
            };
            if name.is_empty() {
                return Err("empty `{}` macro in dir template".to_owned());
            }
            segments.push(DirSegment::Macro(name));
            rest = tail;
        }
        if !rest.is_empty() {
            segments.push(DirSegment::Literal(rest));
        }
        Ok(Self { segments })
    }

    /// Every macro name the template uses, in source order.
    pub fn macros(&self) -> impl Iterator<Item = &'a str> + '_ {
        self.segments.iter().filter_map(|segment| match segment {
            DirSegment::Macro(name) => Some(*name),
            DirSegment::Literal(_) => None,
        })
    }

    /// Expand the template, asking `value` for each macro's text.
    ///
    /// # Errors
    /// Returns a message when a macro has no value, or its value is not a
    /// single path component (empty, `.`, `..`, or containing a separator).
    pub fn expand(&self, value: impl Fn(&str) -> Option<String>) -> Result<String, String> {
        let mut out = String::new();
        for segment in &self.segments {
            match segment {
                DirSegment::Literal(text) => out.push_str(text),
                DirSegment::Macro(name) => {
                    let expanded = value(name).ok_or_else(|| format!("no value for {{{name}}}"))?;
                    validate_component(&expanded)
                        .map_err(|message| format!("{{{name}}}: {message}"))?;
                    out.push_str(&expanded);
                }
            }
        }
        Ok(out)
    }
}

/// Whether `value` is one path component: non-empty, not `.` or `..`, and
/// carrying no `/` or `\`.
///
/// # Errors
/// Returns a message that reports what the value violates.
pub fn validate_component(value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err("empty value".to_owned());
    }
    if value == "." || value == ".." {
        return Err(format!("{} names a directory", crate::config::quote(value)));
    }
    if value.contains('/') || value.contains('\\') {
        return Err("contains a path separator".to_owned());
    }
    Ok(())
}

/// Check one static `dir` template: relative, with known macros.
///
/// # Errors
/// Returns a message for an absolute path or an unknown macro.
pub(super) fn validate_dir(text: &str) -> Result<(), String> {
    validate_relative_dir(text, "dir")?;
    if text.is_empty() {
        return Ok(());
    }
    let template = DirTemplate::parse(text)?;
    for name in template.macros() {
        if name != TEMPLATE_MACRO && !is_add_safe(name) && !is_date_macro(name) {
            return Err(format!("unknown dir macro: {{{name}}}"));
        }
    }
    Ok(())
}

/// Check a `dir` a derive function returned: relative, with no `.` or `..`
/// component. The empty string is the project root, as a static template's is.
/// Whether the path lies under a configured root is the add pipeline's check,
/// against the project it writes into.
///
/// # Errors
/// Returns a message that reports what the path violates.
pub fn validate_derived_dir(text: &str) -> Result<(), String> {
    validate_relative_dir(text, "a derived dir")
}

/// Check a relative directory template: no absolute path, and no empty, `.` or
/// `..` component. The static and derived `dir` forms share this rule, so a
/// path one accepts cannot escape the project through the other.
fn validate_relative_dir(text: &str, noun: &str) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }
    if is_absolute(text) {
        return Err(format!("{noun} must be relative to the project root"));
    }
    let parts: Vec<&str> = text.split(['/', '\\']).collect();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(format!("{noun} holds an empty path component"));
    }
    if parts.iter().any(|part| *part == "." || *part == "..") {
        return Err(format!("{noun} must not hold `.` or `..`"));
    }
    Ok(())
}

/// Whether `text` names an absolute path on any platform, so a config written
/// on one machine cannot smuggle an absolute destination in on another.
fn is_absolute(text: &str) -> bool {
    text.starts_with('/')
        || text.starts_with('\\')
        || Path::new(text).is_absolute()
        || matches!(text.as_bytes(), [drive, b':', ..] if drive.is_ascii_alphabetic())
}

#[cfg(test)]
#[path = "add_hooks_dir_tests.rs"]
mod tests;
