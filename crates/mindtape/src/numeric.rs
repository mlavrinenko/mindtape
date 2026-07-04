//! Numeric-field primitive: scale, classification, resolve and display.
//!
//! The numeric cousin of the [`Legend`](crate::config::Legend) primitive. It
//! owns the machinery shared by the scalar numeric fields (priority, difficulty)
//! so neither reinvents the scale, the numeric value, or the classification
//! layer — and so neither has to depend on the other.
//!
//! A [`NumericField`] is a `[min, max]` scale (`int` or `real`) plus one
//! classification style:
//!
//! - bands — contiguous ranges that tile the scale (int: inclusive and
//!   contiguous; real: half-open, closed last). A value falls in exactly one.
//! - anchors — sparse labelled points. A value at an anchor shows its label;
//!   a value between anchors inherits the nearest lower anchor's glyph and
//!   colour, its label derived as `lower-label + "+"×offset` (values below all
//!   anchors derive upward with `−`).
//!
//! Exactly one style is active: declaring both is a config error.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde::Deserialize;

use crate::config::Requirement;

/// One typed input value supplied to a priority framework (e.g. ICE's `impact`).
///
/// Carries the value as the prelude packager shaped it; the framework's declared
/// input types drive how it feeds the score formula or category predicate.
#[derive(Debug, Clone, PartialEq)]
pub enum InputValue {
    /// A numeric input (fed to the evalexpr context as a float).
    Num(f64),
    /// A boolean input (e.g. Eisenhower's `urgent`).
    Bool(bool),
    /// A string input (e.g. `MoSCoW`'s `level`).
    Text(String),
}

/// A numeric-field value as written in the DSL: a point on the scale, a
/// classification label, or a priority framework dict carrying its inputs.
#[derive(Debug, Clone, PartialEq)]
pub enum NumValue {
    /// A raw number on the scale (e.g. `priority: 4`).
    Num(f64),
    /// A band or anchor label (e.g. `difficulty: "senior"`).
    Label(String),
    /// A framework invocation (e.g. `priority: ice(impact: 8, ...)`): the
    /// framework name plus its named inputs. The score is resolved against the
    /// configured framework registry, never by the numeric field alone.
    Framework {
        /// Framework name, matched against the configured registry.
        name: String,
        /// Named inputs as shaped by the prelude packager.
        inputs: BTreeMap<String, InputValue>,
    },
}

impl NumValue {
    /// Parse a string form back into a value: a parseable number becomes
    /// [`NumValue::Num`], anything else a [`NumValue::Label`]. Framework values
    /// never round-trip through a string and are resolved with config instead.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        text.parse::<f64>()
            .map_or_else(|_| Self::Label(text.to_owned()), Self::Num)
    }

    /// The plain display string: a whole number renders without a fraction. A
    /// framework value falls back to its name (the score needs config to
    /// resolve; callers with config use [`crate::config::PriorityConfig`]).
    #[must_use]
    pub fn as_display(&self) -> String {
        match self {
            Self::Num(num) => format_num(*num),
            Self::Label(label) => label.clone(),
            Self::Framework { name, .. } => name.clone(),
        }
    }
}

/// Render a number without a trailing `.0` when it is whole.
#[must_use]
pub fn format_num(num: f64) -> String {
    if num.fract() == 0.0 {
        // The value is whole; print it as an integer. Bounded by the configured
        // scale, so the conversion cannot meaningfully truncate.
        #[allow(clippy::cast_possible_truncation)]
        let whole = num.round() as i64;
        whole.to_string()
    } else {
        num.to_string()
    }
}

/// Whether the scale carries integers or reals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScaleKind {
    /// Whole numbers; bands are inclusive and contiguous.
    #[default]
    Int,
    /// Real numbers; bands are half-open, the last one closed.
    Real,
}

/// One contiguous labelled range in a band classification.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Band {
    /// Display label.
    pub label: String,
    /// Inclusive lower bound.
    pub min: f64,
    /// Upper bound (inclusive for int and the last real band; else exclusive).
    pub max: f64,
    /// Optional decorative glyph.
    pub glyph: Option<String>,
    /// Optional colour: a named ANSI colour or `#RRGGBB`.
    pub color: Option<String>,
}

/// One labelled point in an anchor classification.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Anchor {
    /// The point on the scale this label names.
    pub value: f64,
    /// Display label.
    pub label: String,
    /// Optional decorative glyph.
    pub glyph: Option<String>,
    /// Optional colour: a named ANSI colour or `#RRGGBB`.
    pub color: Option<String>,
}

/// A classified value resolved into its display parts.
#[derive(Debug, Clone, PartialEq)]
pub struct NumCell {
    /// Display label (an anchor/band label, or a derived `senior+`).
    pub label: String,
    /// Optional decorative glyph.
    pub glyph: Option<String>,
    /// Optional colour spec.
    pub color: Option<String>,
}

/// A numeric scalar field: scale plus one classification style.
///
/// Left opinionated (a usable `0..9` scale, no bands/anchors) rather than
/// collapsed to a degenerate `0..0`: `Config::default()` is called directly
/// (bypassing `parse_config`) by CLI test fixtures that carry an in-range
/// manual priority/difficulty value and rely on `check_in_scale` accepting it
/// with no classification declared. Its scale already agrees with the
/// embedded floor's (`crate::config::EMBEDDED_DEFAULTS`), which additionally
/// supplies the bands/anchors through `parse_config`.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct NumericField {
    /// Whether the scale is integer or real.
    pub kind: ScaleKind,
    /// Inclusive scale minimum.
    pub min: f64,
    /// Inclusive scale maximum.
    pub max: f64,
    /// Band classification (mutually exclusive with `anchors`).
    pub bands: Vec<Band>,
    /// Anchor classification (mutually exclusive with `bands`).
    pub anchors: Vec<Anchor>,
    /// Whether the field must be set.
    pub required: Requirement,
}

impl Default for NumericField {
    fn default() -> Self {
        Self {
            kind: ScaleKind::Int,
            min: 0.0,
            max: 9.0,
            bands: Vec::new(),
            anchors: Vec::new(),
            required: Requirement::No,
        }
    }
}

impl NumericField {
    /// Resolve a written value to its numeric point: a number passes through;
    /// a label maps via its band (representative = lower bound) or anchor.
    #[must_use]
    pub fn resolve(&self, value: &NumValue) -> Option<f64> {
        match value {
            NumValue::Num(num) => Some(*num),
            NumValue::Label(label) => self.label_value(label),
            // A framework score needs the framework registry; the numeric field
            // alone cannot resolve it. `PriorityConfig::resolve` handles this.
            NumValue::Framework { .. } => None,
        }
    }

    fn label_value(&self, label: &str) -> Option<f64> {
        if let Some(anchor) = self.anchors.iter().find(|anchor| anchor.label == label) {
            return Some(anchor.value);
        }
        self.bands
            .iter()
            .find(|band| band.label == label)
            .map(|band| band.min)
    }

    /// Classify a resolved point into its display cell, or `None` when the
    /// field declares no classification.
    #[must_use]
    pub fn classify(&self, point: f64) -> Option<NumCell> {
        if !self.anchors.is_empty() {
            return Some(self.classify_anchor(point));
        }
        self.bands
            .iter()
            .find(|band| self.in_band(band, point))
            .map(|band| NumCell {
                label: band.label.clone(),
                glyph: band.glyph.clone(),
                color: band.color.clone(),
            })
    }

    /// Resolve and classify a written value in one step.
    #[must_use]
    pub fn cell(&self, value: &NumValue) -> Option<NumCell> {
        self.classify(self.resolve(value)?)
    }

    fn in_band(&self, band: &Band, point: f64) -> bool {
        if point < band.min {
            return false;
        }
        match self.kind {
            ScaleKind::Int => point <= band.max,
            // Half-open, except the band closing the scale is inclusive.
            ScaleKind::Real => point < band.max || (point == band.max && band.max == self.max),
        }
    }

    fn classify_anchor(&self, point: f64) -> NumCell {
        let mut anchors: Vec<&Anchor> = self.anchors.iter().collect();
        anchors.sort_by(|left, right| {
            left.value
                .partial_cmp(&right.value)
                .unwrap_or(Ordering::Equal)
        });
        if let Some(exact) = anchors.iter().find(|anchor| anchor.value == point) {
            return anchor_cell(exact, exact.label.clone());
        }
        if let Some(lower) = anchors.iter().rev().find(|anchor| anchor.value < point) {
            let label = derive_label(&lower.label, '+', point - lower.value);
            return anchor_cell(lower, label);
        }
        match anchors.first() {
            Some(lowest) => {
                let label = derive_label(&lowest.label, '\u{2212}', lowest.value - point);
                anchor_cell(lowest, label)
            }
            None => NumCell {
                label: format_num(point),
                glyph: None,
                color: None,
            },
        }
    }

    /// Validate this field's scale and classification, returning one message per
    /// problem (empty = valid). `field` names the field for the messages.
    #[must_use]
    pub fn validate(&self, field: &str) -> Vec<String> {
        let mut out = Vec::new();
        if self.min >= self.max {
            out.push(format!(
                "{field}: min {} must be below max {}",
                format_num(self.min),
                format_num(self.max),
            ));
        }
        if !self.bands.is_empty() && !self.anchors.is_empty() {
            out.push(format!("{field}: declare bands or anchors, not both"));
        }
        if !self.bands.is_empty() {
            self.validate_bands(field, &mut out);
        }
        if !self.anchors.is_empty() {
            self.validate_anchors(field, &mut out);
        }
        out
    }

    fn validate_bands(&self, field: &str, out: &mut Vec<String>) {
        let mut bands: Vec<&Band> = self.bands.iter().collect();
        bands.sort_by(|left, right| left.min.partial_cmp(&right.min).unwrap_or(Ordering::Equal));
        if let Some(first) = bands.first()
            && first.min != self.min
        {
            out.push(format!(
                "{field}: bands must start at {}",
                format_num(self.min)
            ));
        }
        if let Some(last) = bands.last()
            && last.max != self.max
        {
            out.push(format!(
                "{field}: bands must end at {}",
                format_num(self.max)
            ));
        }
        for band in &bands {
            if band.min > band.max {
                out.push(format!("{field}: band `{}` has min above max", band.label));
            }
        }
        let step = if self.kind == ScaleKind::Int {
            1.0
        } else {
            0.0
        };
        for pair in bands.windows(2) {
            if let [lower, upper] = pair
                && upper.min != lower.max + step
            {
                out.push(format!(
                    "{field}: bands `{}` and `{}` are not contiguous",
                    lower.label, upper.label
                ));
            }
        }
    }

    fn validate_anchors(&self, field: &str, out: &mut Vec<String>) {
        if !self.anchors.iter().any(|anchor| anchor.value == self.min) {
            out.push(format!(
                "{field}: requires an anchor at {}",
                format_num(self.min)
            ));
        }
        for anchor in &self.anchors {
            if anchor.value < self.min || anchor.value > self.max {
                out.push(format!(
                    "{field}: anchor `{}` at {} is out of range",
                    anchor.label,
                    format_num(anchor.value)
                ));
            }
        }
        let mut values: Vec<f64> = self.anchors.iter().map(|anchor| anchor.value).collect();
        values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
        for pair in values.windows(2) {
            if let [lower, upper] = pair
                && lower == upper
            {
                out.push(format!(
                    "{field}: duplicate anchor at {}",
                    format_num(*lower)
                ));
            }
        }
    }
}

fn anchor_cell(anchor: &Anchor, label: String) -> NumCell {
    NumCell {
        label,
        glyph: anchor.glyph.clone(),
        color: anchor.color.clone(),
    }
}

/// Build a derived label `base` followed by `steps` copies of `symbol`.
fn derive_label(base: &str, symbol: char, steps: f64) -> String {
    let rounded = steps.max(0.0).round();
    // `rounded` is a small non-negative whole number (a step offset on the
    // scale); the conversion is exact for the configured ranges.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let count = rounded as usize;
    let suffix = String::from(symbol).repeat(count);
    format!("{base}{suffix}")
}

#[cfg(test)]
#[path = "numeric_tests.rs"]
mod tests;
