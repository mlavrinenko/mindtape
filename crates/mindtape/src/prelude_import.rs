//! Rewrite the prelude version an import names.
//!
//! A task file imports `@local/mindtape` by version, and a version this `mt`
//! no longer reads fails the whole project. This module finds those imports on
//! the parse plane and reports the one edit a fixer may make: replace the
//! version bytes of an older unsupported spec with
//! [`crate::PRELUDE_VERSION`]. Every other byte, including the quotes, the
//! `#import`, the imported items and the whitespace around them, is preserved.
//!
//! Only a string-literal source of a `#import` counts. [`typst_syntax`] gives
//! the exact byte span, so a `#include`, a comment, a computed source or
//! another package never matches. The classification keeps every decision in
//! one place: [`PreludeImportAction::Current`] is silent,
//! [`PreludeImportAction::Keep`] is a supported old pin,
//! [`PreludeImportAction::Rewrite`] carries the byte range, and
//! [`PreludeImportAction::Refuse`] says why nothing is safe to write.

use std::ops::Range;
use std::path::Path;

use typst_syntax::ast::AstNode;
use typst_syntax::package::{PackageManifest, PackageVersion};
use typst_syntax::{LinkedNode, ast};

use crate::config::Config;
use crate::loader::{PRELUDE_VERSION, prelude_is_older, prelude_supports};

/// The package every matched import names.
const PACKAGE: &str = "@local/mindtape";

/// [`PACKAGE`] with the colon that starts its version.
const PREFIX: &str = "@local/mindtape:";

/// What a fixer does with one `@local/mindtape` import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreludeImportAction {
    /// The current version, which a report does not name.
    Current,
    /// A supported older version, kept and reported.
    Keep,
    /// An unsupported older version, rewritten to [`crate::PRELUDE_VERSION`].
    Rewrite,
    /// A version `mt fix` will not rewrite.
    Refuse(PreludeRefusal),
}

/// Why `mt fix` leaves an import alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreludeRefusal {
    /// Newer than this binary reads: upgrade `mt` instead.
    Newer,
    /// Not a package spec `mt` can parse.
    Malformed,
    /// The version a project-local pin declares.
    Pinned,
}

/// One `@local/mindtape` import found in a source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreludeImport {
    /// The version as the literal spells it, absent for a malformed spec.
    pub version: Option<String>,
    /// The whole unquoted spec.
    pub spec: String,
    /// What `mt fix` does with it.
    pub action: PreludeImportAction,
    /// The version bytes to replace, exactly when the action rewrites.
    pub range: Option<Range<usize>>,
}

/// The `@local/mindtape` imports in `source`, in document order.
///
/// `pinned` is the version a project-local `@local/mindtape` pin declares, if
/// any. An import naming that same unsupported version is refused rather than
/// rewritten: the pin, not the binary's embedded copy, is what that project
/// means the import to read.
#[must_use]
pub fn prelude_imports(source: &str, pinned: Option<&str>) -> Vec<PreludeImport> {
    let pinned = pinned.and_then(|version| version.parse::<PackageVersion>().ok());
    let root = typst_syntax::parse(source);
    let mut out = Vec::new();
    walk(&LinkedNode::new(&root), source, pinned.as_ref(), &mut out);
    out
}

/// Walk `node` and every descendant, collecting each literal import.
fn walk(
    node: &LinkedNode<'_>,
    source: &str,
    pinned: Option<&PackageVersion>,
    out: &mut Vec<PreludeImport>,
) {
    if let Some(import) = node.get().cast::<ast::ModuleImport>()
        && let Some(found) = import_at(node, import, source, pinned)
    {
        out.push(found);
    }
    for child in node.children() {
        walk(&child, source, pinned, out);
    }
}

/// The import `node` holds, or `None` when its source is not a string literal
/// naming exactly [`PACKAGE`].
fn import_at(
    node: &LinkedNode<'_>,
    import: ast::ModuleImport<'_>,
    source: &str,
    pinned: Option<&PackageVersion>,
) -> Option<PreludeImport> {
    let ast::Expr::Str(string) = import.source() else {
        return None;
    };
    let literal = node
        .children()
        .find(|child| std::ptr::eq(child.get(), string.to_untyped()))?;
    let range = literal.range();
    let raw = source.get(range.clone())?;
    let inner = raw.strip_prefix('"')?.strip_suffix('"')?;
    let spec = string.get();
    if spec != PACKAGE && !spec.starts_with(PREFIX) {
        return None;
    }
    let Some(raw_version) = inner.strip_prefix(PREFIX) else {
        return Some(refused(inner, PreludeRefusal::Malformed));
    };
    if inner != spec.as_str() || raw_version.is_empty() {
        return Some(refused(inner, PreludeRefusal::Malformed));
    }
    let Some(version) = raw_version.parse::<PackageVersion>().ok() else {
        return Some(refused(inner, PreludeRefusal::Malformed));
    };
    Some(classify(inner, raw_version, version, range, pinned))
}

/// The import for a parsed spec, with the version span only when it rewrites.
fn classify(
    spec: &str,
    raw_version: &str,
    version: PackageVersion,
    literal: Range<usize>,
    pinned: Option<&PackageVersion>,
) -> PreludeImport {
    let text = version.to_string();
    let current = PRELUDE_VERSION.parse::<PackageVersion>().ok();
    let action = if prelude_supports(&text) {
        if current.as_ref() == Some(&version) {
            PreludeImportAction::Current
        } else {
            PreludeImportAction::Keep
        }
    } else if prelude_is_older(&text) {
        if pinned == Some(&version) {
            PreludeImportAction::Refuse(PreludeRefusal::Pinned)
        } else {
            PreludeImportAction::Rewrite
        }
    } else {
        PreludeImportAction::Refuse(PreludeRefusal::Newer)
    };
    let range = (action == PreludeImportAction::Rewrite).then(|| {
        let start = literal.start + 1 + PREFIX.len();
        start..start + raw_version.len()
    });
    PreludeImport {
        version: Some(raw_version.to_owned()),
        spec: spec.to_owned(),
        action,
        range,
    }
}

/// A malformed spec: there is no version span to edit.
fn refused(spec: &str, reason: PreludeRefusal) -> PreludeImport {
    PreludeImport {
        version: None,
        spec: spec.to_owned(),
        action: PreludeImportAction::Refuse(reason),
        range: None,
    }
}

/// Apply every rewrite in `imports` to `source`.
///
/// Returns the rewritten source, or `None` when an edit cannot be applied —
/// which the callers treat as a refusal before anything is written.
#[must_use]
pub fn apply_rewrites(source: &str, imports: &[PreludeImport]) -> Option<String> {
    let edits: Vec<typst_edit::Edit> = imports
        .iter()
        .filter(|import| import.action == PreludeImportAction::Rewrite)
        .filter_map(|import| import.range.clone())
        .map(|range| typst_edit::Edit::new(range, PRELUDE_VERSION))
        .collect();
    typst_edit::apply(source, edits).ok()
}

/// The version a project-local `@local/mindtape` pin declares, if the project
/// has one and its manifest parses.
#[must_use]
pub fn project_prelude_pin_version(root: &Path, config: &Config) -> Option<String> {
    let pin = config
        .typst
        .local_packages
        .iter()
        .rev()
        .find(|package| package.name == "mindtape")?;
    let text = std::fs::read_to_string(root.join(&pin.path).join("typst.toml")).ok()?;
    let manifest: PackageManifest = toml::from_str(&text).ok()?;
    Some(manifest.package.version.to_string())
}

#[cfg(test)]
#[path = "prelude_import_tests.rs"]
mod tests;
