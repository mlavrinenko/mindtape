//! Field-display seam: `Field` trait and generic cell rendering.
//!
//! This module sits between the terminal-discipline primitives
//! ([`super::color`], [`super::glyph`], [`super::width`]) and the concrete
//! field families (legend, numeric). It defines:
//!
//! - [`CellSpec`] — the rendered value in text/glyph/colour form.
//! - [`render_cell`] — lays out a `CellSpec` into a display string of a fixed
//!   column width, honouring glyph mode and colour activation.
//! - [`Field`] — the trait every displayable task field implements.

use std::borrow::Cow;

use anstyle::Style;
use mindtape::config::Config;
use mindtape::model::Task;

use super::color::Painter;
use super::glyph::GlyphMode;
use super::width::{display_width, pad_to_width, truncate_to_width};

// --- CellSpec ----------------------------------------------------------------

/// A field value resolved into its displayable parts.
///
/// The three parts are independent: a field may have text only, glyph only, or
/// all three. The [`render_cell`] function combines them according to the active
/// [`GlyphMode`] and [`DensityMode`].
#[derive(Debug, Clone, PartialEq)]
pub struct CellSpec {
    /// Primary text label (always rendered in [`DensityMode::Full`]).
    pub text: String,
    /// Optional decorative glyph (emoji or ASCII). Shown in
    /// [`DensityMode::Full`] and [`DensityMode::GlyphOnly`] when the resolved
    /// [`GlyphMode`] is not [`GlyphMode::None`].
    pub glyph: Option<String>,
    /// Optional colour to apply to the entire rendered cell.
    pub color: Option<Style>,
}

// --- DensityMode -------------------------------------------------------------

/// How much information to render into a column cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DensityMode {
    /// Show glyph (if any) followed by text.
    #[default]
    Full,
    /// Show only the glyph; suppress text. Falls back to text when no glyph is
    /// available.
    GlyphOnly,
}

// --- render_cell -------------------------------------------------------------

/// Lay out a [`CellSpec`] into a string of exactly `column_width` display
/// cells.
///
/// Layout rules:
/// 1. Determine the raw content string from `spec`, `glyph_mode`, and `density`.
/// 2. Truncate or pad the content to `column_width` display cells.
/// 3. Wrap in ANSI colour sequences when `painter` is enabled and `spec.color`
///    is set (colour is applied after width layout so ANSI bytes don't confuse
///    the width accounting).
///
/// `column_width == 0` returns an empty string.
#[must_use]
pub fn render_cell(
    spec: &CellSpec,
    glyph_mode: GlyphMode,
    density: DensityMode,
    painter: Painter,
    column_width: usize,
) -> String {
    if column_width == 0 {
        return String::new();
    }

    // 1. Build the unstyled content string.
    let content = build_content(spec, glyph_mode, density);

    // 2. Fit to column width.
    let fitted = fit_to_width(&content, column_width);

    // 3. Apply colour.
    match spec.color {
        Some(style) => painter.paint(&fitted, style),
        None => fitted,
    }
}

/// Build the plain (unstyled) content string from the spec's parts.
fn build_content(spec: &CellSpec, glyph_mode: GlyphMode, density: DensityMode) -> String {
    let show_glyph = glyph_mode != GlyphMode::None;

    match density {
        DensityMode::GlyphOnly => match (show_glyph, spec.glyph.as_deref()) {
            (true, Some(g)) => g.to_owned(),
            // No glyph available or glyphs suppressed — fall back to text.
            _ => spec.text.clone(),
        },
        DensityMode::Full => match (show_glyph, spec.glyph.as_deref()) {
            (true, Some(g)) => format!("{g} {}", spec.text),
            _ => spec.text.clone(),
        },
    }
}

/// Truncate-or-pad `content` to exactly `width` display cells.
fn fit_to_width(content: &str, width: usize) -> String {
    if display_width(content) > width {
        truncate_to_width(content, width)
    } else {
        pad_to_width(content, width)
    }
}

/// The plain (unstyled) string a cell renders to, before width fitting.
#[must_use]
pub fn cell_content(spec: &CellSpec, glyph_mode: GlyphMode, density: DensityMode) -> String {
    build_content(spec, glyph_mode, density)
}

/// Display width of a cell's content, for column sizing.
#[must_use]
pub fn content_width(spec: &CellSpec, glyph_mode: GlyphMode, density: DensityMode) -> usize {
    display_width(&build_content(spec, glyph_mode, density))
}

// --- Field trait -------------------------------------------------------------

/// The seam every displayable task field renders through.
///
/// Implementors live in the field-family tasks (`field-legend-glyphs`,
/// `numeric-field`). This trait owns only the interface, not the per-field
/// config lookups.
pub trait Field {
    /// Stable column name used as a table header.
    const NAME: &'static str;

    /// Extract the raw string value(s) for this field from `task`.
    ///
    /// Single-valued fields return 0 or 1 items. Multi-valued fields (e.g.
    /// `tags`) return N items.
    fn values<'a>(task: &'a Task) -> Vec<Cow<'a, str>>;

    /// Resolve a single raw `value` to a [`CellSpec`] using `config` for
    /// legend / classification lookups.
    ///
    /// Concrete per-field logic (colour maps, glyph legends) is implemented
    /// by each field type; the default returns a plain-text spec.
    fn cell_spec(value: &str, config: &Config) -> CellSpec {
        let _ = config;
        CellSpec {
            text: value.to_owned(),
            glyph: None,
            color: None,
        }
    }
}

// --- Tests -------------------------------------------------------------------

#[cfg(test)]
#[path = "field_tests.rs"]
mod tests;
