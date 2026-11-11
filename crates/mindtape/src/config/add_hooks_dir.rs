//! `dir` template parsing and validation, split out of `add_hooks.rs` for the
//! file-size gate.
//!
//! A `dir` is project-root-relative text with `{macro}` placeholders. A static
//! template may name `{template}` and any add-safe scalar field; a value a
//! derive function returns is a plain path and skips macro expansion, but must
//! still stay relative and free of `.`/`..`.

use std::path::Path;

use crate::config::add_hooks::is_add_safe;

/// The one non-field macro a `dir` template takes.
pub(super) const TEMPLATE_MACRO: &str = "template";

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
            let Some(close) = after.find('}') else {
                return Err("unclosed `{` in dir template".to_owned());
            };
            let Some(name) = after.get(1..close) else {
                return Err("unclosed `{` in dir template".to_owned());
            };
            if name.is_empty() {
                return Err("empty `{}` macro in dir template".to_owned());
            }
            segments.push(DirSegment::Macro(name));
            rest = after.get(close + 1..).unwrap_or_default();
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
/// Returns a message naming what the value violates.
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
    if text.is_empty() {
        return Ok(());
    }
    if is_absolute(text) {
        return Err("dir must be relative to the project root".to_owned());
    }
    let template = DirTemplate::parse(text)?;
    for name in template.macros() {
        if name != TEMPLATE_MACRO && !is_add_safe(name) {
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
/// Returns a message naming what the path violates.
pub fn validate_derived_dir(text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }
    if is_absolute(text) {
        return Err("a derived dir must be relative to the project root".to_owned());
    }
    let parts: Vec<&str> = text.split(['/', '\\']).collect();
    if parts.iter().any(|part| part.is_empty()) {
        return Err("a derived dir holds an empty path component".to_owned());
    }
    if parts.iter().any(|part| *part == "." || *part == "..") {
        return Err("a derived dir must not hold `.` or `..`".to_owned());
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
