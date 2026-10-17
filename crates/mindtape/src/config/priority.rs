//! `[tasks.priority]`: the priority scale and its scoring frameworks, with the
//! hand-written `Deserialize` that keeps unknown keys an error.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};

use crate::config::required::Requirement;
use crate::framework::Framework;
use crate::model::numeric::{Anchor, Band, NumericField, ScaleKind};

/// The priority scale and its scoring frameworks.
///
/// It takes the same scale, bands and anchors as `[tasks.difficulty]`. When
/// `framework` names a framework, `mt` computes a task's priority from that
/// framework's inputs. `allow-manual` decides whether a raw integer is still
/// accepted. `frameworks` holds the built-in frameworks merged with any the
/// project declares.
// `Deserialize` is hand-written: serde cannot combine `flatten` with
// `deny_unknown_fields`, so `PriorityRaw` spells the scale keys out.
#[derive(Debug, Clone, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PriorityConfig {
    /// Scale and band classification (shared numeric-field machinery).
    #[serde(flatten)]
    pub field: NumericField,
    /// Name of the framework that computes priority, a key of `frameworks`.
    /// Absent means priority is a raw number.
    pub framework: Option<String>,
    /// Whether a raw integer is accepted while a framework is active.
    pub allow_manual: bool,
    /// Scoring frameworks by name: the built-in ones merged with any declared
    /// here.
    pub frameworks: BTreeMap<String, Framework>,
}

/// The keys of `[tasks.priority]` with the scale keys inlined, so an unknown
/// key is an error.
#[derive(Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
struct PriorityRaw {
    kind: ScaleKind,
    min: f64,
    max: f64,
    bands: Vec<Band>,
    anchors: Vec<Anchor>,
    required: Requirement,
    example: Option<f64>,
    framework: Option<String>,
    allow_manual: bool,
    frameworks: BTreeMap<String, Framework>,
}

impl Default for PriorityRaw {
    fn default() -> Self {
        let PriorityConfig {
            field,
            framework,
            allow_manual,
            frameworks,
        } = PriorityConfig::default();
        Self {
            kind: field.kind,
            min: field.min,
            max: field.max,
            bands: field.bands,
            anchors: field.anchors,
            required: field.required,
            example: field.example,
            framework,
            allow_manual,
            frameworks,
        }
    }
}

impl<'de> Deserialize<'de> for PriorityConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = PriorityRaw::deserialize(deserializer)?;
        Ok(Self {
            field: NumericField {
                kind: raw.kind,
                min: raw.min,
                max: raw.max,
                bands: raw.bands,
                anchors: raw.anchors,
                required: raw.required,
                example: raw.example,
            },
            framework: raw.framework,
            allow_manual: raw.allow_manual,
            frameworks: raw.frameworks,
        })
    }
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            field: NumericField::default(),
            framework: None,
            allow_manual: true,
            frameworks: BTreeMap::new(),
        }
    }
}
