//! Typed relation edges: the `links:` field's kind registry and its config.
//!
//! Scaffold for the typed-relation-edges task. The `[tasks.links]` /
//! `[[tasks.links.kinds]]` config surface lives here; the edge model
//! (`Link { task, kind, note }`), the immutable built-in kind table, and the
//! per-kind validation land under that task. Built-in kinds ship in code and
//! cannot be overridden or removed — config only adds custom kinds on top.

use serde::Deserialize;

/// `[tasks.links]` — the typed-edge kind registry.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LinksConfig {
    /// User-declared custom edge kinds, layered over the immutable built-ins.
    pub kinds: Vec<LinkKind>,
}

/// `[[tasks.links.kinds]]` — one project/consumer-declared custom edge kind.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LinkKind {
    /// Kind name, matched against an edge's `kind`.
    pub name: String,
    /// Cardinality cap; `None` = unbounded.
    pub max: Option<u32>,
    /// Note requirement for edges of this kind.
    pub note: NoteRule,
    /// Whether edges of this kind gate readiness.
    pub blocking: bool,
    /// Whether edges of this kind must stay acyclic.
    pub acyclic: bool,
}

impl Default for LinkKind {
    fn default() -> Self {
        Self {
            name: String::new(),
            max: None,
            note: NoteRule::Optional,
            blocking: false,
            acyclic: false,
        }
    }
}

/// Note requirement for an edge kind.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NoteRule {
    /// A non-empty note is mandatory (the `related` escape hatch).
    Required,
    /// A note is allowed but not required (the default).
    #[default]
    Optional,
    /// A note is rejected.
    Forbidden,
}
