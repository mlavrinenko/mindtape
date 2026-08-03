//! Folder-derived tags: virtual tags injected from a task's location under its
//! root. Pure (no IO) — the loader supplies the path relative to the root dir.
//!
//! A [`RootDeriver`] is built once per root (it compiles the gating globs),
//! then queried per file. Tags are the union of the root name plus the
//! configured collectors over the intervening folder segments.

use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::config::{Collector, Config, DeriveConfig, DeriveTags, RootSpec};

/// Derivation policy resolved for one root, ready to query per file.
pub struct RootDeriver<'a> {
    enabled: bool,
    name: String,
    /// Gate matcher; `None` = every file under the root.
    matcher: Option<GlobSet>,
    global: &'a DeriveConfig,
}

impl<'a> RootDeriver<'a> {
    /// Resolve `root` against the project-wide `global` policy.
    #[must_use]
    pub fn new(root: &RootSpec, global: &'a DeriveConfig) -> Self {
        let (enabled, globs) = resolve(root, global);
        Self {
            enabled,
            name: root.name(),
            matcher: build_globset(&globs),
            global,
        }
    }

    /// Derived tags for a file at `rel_within_root` (relative to the root dir).
    /// Empty when derivation is off for this root or the file is gated out.
    #[must_use]
    pub fn tags_for(&self, rel_within_root: &Path) -> Vec<String> {
        if !self.enabled {
            return Vec::new();
        }
        if let Some(set) = &self.matcher
            && !set.is_match(rel_within_root)
        {
            return Vec::new();
        }
        collect(&self.name, &folder_segments(rel_within_root), self.global)
    }
}

/// Resolve the per-file gate into `(enabled, globs)`. Empty globs with `enabled`
/// means every file; non-empty globs gate to matches.
fn resolve(root: &RootSpec, global: &DeriveConfig) -> (bool, Vec<String>) {
    match root.derive_tags() {
        None => (global.enabled, Vec::new()),
        Some(DeriveTags::Enabled(flag)) => (*flag, Vec::new()),
        Some(DeriveTags::Glob(glob)) => (true, vec![glob.clone()]),
        Some(DeriveTags::Globs(globs)) => (true, globs.clone()),
    }
}

fn build_globset(globs: &[String]) -> Option<GlobSet> {
    if globs.is_empty() {
        return None;
    }
    let mut builder = GlobSetBuilder::new();
    for glob in globs {
        if let Ok(compiled) = Glob::new(glob) {
            builder.add(compiled);
        }
    }
    builder.build().ok()
}

/// The root name plus collector output over `segments`, deduped in
/// root → outermost → innermost order.
fn collect(root_name: &str, segments: &[String], cfg: &DeriveConfig) -> Vec<String> {
    let slug = |text: &str| -> String {
        if cfg.slugify {
            slugify(text)
        } else {
            text.to_owned()
        }
    };
    let mut out = Vec::new();
    push_unique(&mut out, slug(root_name));
    if cfg.collect.contains(&Collector::Segments) {
        for seg in segments {
            push_unique(&mut out, slug(seg));
        }
    }
    if cfg.collect.contains(&Collector::Breadcrumbs) {
        let mut chain: Vec<String> = Vec::new();
        if cfg.breadcrumb_root {
            chain.push(slug(root_name));
        }
        for seg in segments {
            chain.push(slug(seg));
            push_unique(&mut out, chain.join(&cfg.separator));
        }
    }
    out
}

fn push_unique(out: &mut Vec<String>, tag: String) {
    if !tag.is_empty() && !out.contains(&tag) {
        out.push(tag);
    }
}

/// Folder components between the root and the file (the file name excluded).
fn folder_segments(rel: &Path) -> Vec<String> {
    rel.parent()
        .unwrap_or(Path::new(""))
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(seg) => Some(seg.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}

/// Nesting depth of a file under its root: the number of folder components
/// between the root and the file (the file name excluded). A file sitting
/// directly under the root is depth 0, one in a sub-folder is depth 1, and so
/// on. Reuses [`folder_segments`] so the folder count and the derived-tag
/// segments can never disagree.
#[must_use]
pub fn depth(rel: &Path) -> u8 {
    // A path can exceed 255 segments only in a pathological tree; clamp rather
    // than truncate silently.
    u8::try_from(folder_segments(rel).len()).unwrap_or(u8::MAX)
}

/// Lowercase, collapse non-alphanumeric runs to `-`, trim leading/trailing `-`.
fn slugify(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            out.extend(ch.to_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

/// Validate that every per-root gating glob compiles. Returns one message per
/// bad glob, prefixed with the offending root path.
#[must_use]
pub fn validate_root_globs(config: &Config) -> Vec<String> {
    let mut problems = Vec::new();
    for root in &config.tasks.roots {
        let globs = match root.derive_tags() {
            Some(DeriveTags::Glob(glob)) => vec![glob.clone()],
            Some(DeriveTags::Globs(globs)) => globs.clone(),
            _ => Vec::new(),
        };
        for glob in globs {
            if let Err(err) = Glob::new(&glob) {
                problems.push(format!("tasks.roots[{}]: invalid glob: {err}", root.path()));
            }
        }
    }
    problems
}

#[cfg(test)]
#[path = "derive_tests.rs"]
mod tests;
