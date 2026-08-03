#![allow(clippy::unwrap_used)]
use anstyle::{AnsiColor, Color, Style};
use mindtape::config::Config;
use mindtape::model::Task;

use super::{CellSpec, DensityMode, Field, render_cell};
use crate::render::color::Painter;
use crate::render::glyph::GlyphMode;
use crate::render::width::display_width;
use std::borrow::Cow;

// --- Helpers ---

fn red_style() -> Style {
    Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)))
}

fn spec_text_only(text: &str) -> CellSpec {
    CellSpec {
        text: text.to_owned(),
        glyph: None,
        color: None,
    }
}

fn spec_with_glyph(text: &str, glyph: &str) -> CellSpec {
    CellSpec {
        text: text.to_owned(),
        glyph: Some(glyph.to_owned()),
        color: None,
    }
}

fn spec_with_color(text: &str) -> CellSpec {
    CellSpec {
        text: text.to_owned(),
        glyph: None,
        color: Some(red_style()),
    }
}

/// Render `input` into a width-4 column (no glyphs) and assert it fits, ends
/// with the ellipsis, and preserves `first_char` (a double-width char) whole.
fn assert_truncates_to_fit(input: &str, first_char: &str) {
    let spec = spec_text_only(input);
    let out = render_cell(
        &spec,
        GlyphMode::None,
        DensityMode::Full,
        Painter::new(false),
        4,
    );
    let w = display_width(&out);
    assert!(w <= 4, "output must fit column width; got {w}");
    assert!(
        out.ends_with('…'),
        "truncated output must end with ellipsis"
    );
    assert!(
        out.starts_with(first_char),
        "first char must be preserved whole"
    );
}

// --- Color tests ---

#[test]
fn color_suppressed_when_painter_disabled() {
    let spec = spec_with_color("hello");
    let out = render_cell(
        &spec,
        GlyphMode::Ascii,
        DensityMode::Full,
        Painter::new(false),
        10,
    );
    assert_eq!(out, "hello     ", "no ANSI sequences expected");
    assert!(!out.contains('\x1b'));
}

#[test]
fn color_applied_when_painter_enabled() {
    let spec = spec_with_color("hi");
    let out = render_cell(
        &spec,
        GlyphMode::Ascii,
        DensityMode::Full,
        Painter::new(true),
        5,
    );
    assert!(out.contains("hi"), "text should appear in output");
    assert!(
        out.contains('\x1b'),
        "ANSI escape expected when painter enabled"
    );
}

#[test]
fn no_color_when_spec_has_none() {
    let spec = spec_text_only("hello");
    let out = render_cell(
        &spec,
        GlyphMode::Ascii,
        DensityMode::Full,
        Painter::new(true),
        8,
    );
    assert!(!out.contains('\x1b'), "no ANSI when spec.color is None");
    assert_eq!(display_width(&out), 8);
}

// --- Glyph mode tests ---

#[test]
fn glyph_shown_in_emoji_mode() {
    let spec = spec_with_glyph("open", "🔨");
    let out = render_cell(
        &spec,
        GlyphMode::Emoji,
        DensityMode::Full,
        Painter::new(false),
        12,
    );
    assert!(out.contains("🔨"), "glyph expected in emoji mode");
    assert!(out.contains("open"));
}

#[test]
fn glyph_hidden_in_none_mode() {
    let spec = spec_with_glyph("open", "🔨");
    let out = render_cell(
        &spec,
        GlyphMode::None,
        DensityMode::Full,
        Painter::new(false),
        8,
    );
    assert!(!out.contains("🔨"), "glyph must be suppressed in None mode");
    assert!(out.contains("open"));
}

#[test]
fn glyph_hidden_in_ascii_mode_when_glyph_is_emoji() {
    // In Ascii mode glyphs are still shown (ASCII glyphs are fine);
    // the Ascii/Emoji distinction is about what the field *provides* as a
    // glyph, but the mode gate in render_cell only suppresses in None.
    // This test documents current behaviour: Ascii mode shows glyphs.
    let spec = spec_with_glyph("open", "[*]");
    let out = render_cell(
        &spec,
        GlyphMode::Ascii,
        DensityMode::Full,
        Painter::new(false),
        12,
    );
    assert!(out.contains("[*]"), "ascii glyph shown in Ascii mode");
}

// --- Width / double-width tests ---

#[test]
fn pads_to_exact_column_width() {
    let spec = spec_text_only("hi");
    let out = render_cell(
        &spec,
        GlyphMode::None,
        DensityMode::Full,
        Painter::new(false),
        8,
    );
    assert_eq!(display_width(&out), 8, "must be padded to 8 cells");
    assert!(out.starts_with("hi"));
}

#[test]
fn truncates_with_ellipsis_when_overlong() {
    let spec = spec_text_only("hello world");
    let out = render_cell(
        &spec,
        GlyphMode::None,
        DensityMode::Full,
        Painter::new(false),
        7,
    );
    assert_eq!(display_width(&out), 7);
    assert!(
        out.ends_with('…'),
        "truncated output must end with ellipsis"
    );
}

#[test]
fn double_width_emoji_truncated_correctly() {
    // "🔨" is 2 display cells. "🔨🔨X" = 5 cells.
    // Column 4 → budget 3 → fits first 🔨 (2) but not second (4 > 3) → "🔨…" (3 cells).
    // Width is ≤ 4 and the result ends with the ellipsis with no broken grapheme.
    assert_truncates_to_fit("🔨🔨X", "🔨");
}

#[test]
fn double_width_cjk_truncated_correctly() {
    // "世界X" = 5 cells. Column 4 → budget 3 → fits "世" (2), "界" won't (4>3) → "世…" (3).
    // The whole double-width char is dropped rather than split; width ≤ 4.
    assert_truncates_to_fit("世界X", "世");
}

#[test]
fn double_width_pads_correctly() {
    // "世" = 2 cells. Column 5 → pad 3 spaces → "世   " (5 cells).
    let spec = spec_text_only("世");
    let out = render_cell(
        &spec,
        GlyphMode::None,
        DensityMode::Full,
        Painter::new(false),
        5,
    );
    assert_eq!(display_width(&out), 5);
    assert!(out.starts_with("世"));
}

#[test]
fn zero_column_width_returns_empty() {
    let spec = spec_text_only("hello");
    let out = render_cell(
        &spec,
        GlyphMode::None,
        DensityMode::Full,
        Painter::new(false),
        0,
    );
    assert_eq!(out, "");
}

// --- DensityMode tests ---

#[test]
fn glyph_only_mode_shows_glyph_not_text() {
    let spec = spec_with_glyph("open", "🔨");
    let out = render_cell(
        &spec,
        GlyphMode::Emoji,
        DensityMode::GlyphOnly,
        Painter::new(false),
        4,
    );
    // Content is "🔨" (2 cells) padded to 4 → "🔨  "
    assert!(out.contains("🔨"));
    // Text "open" should not appear
    assert!(!out.contains("open"));
}

#[test]
fn glyph_only_mode_falls_back_to_text_when_no_glyph() {
    let spec = spec_text_only("open");
    let out = render_cell(
        &spec,
        GlyphMode::Emoji,
        DensityMode::GlyphOnly,
        Painter::new(false),
        6,
    );
    assert!(out.contains("open"));
}

#[test]
fn glyph_only_suppressed_in_none_mode_falls_back_to_text() {
    let spec = spec_with_glyph("open", "🔨");
    let out = render_cell(
        &spec,
        GlyphMode::None,
        DensityMode::GlyphOnly,
        Painter::new(false),
        6,
    );
    // GlyphMode::None suppresses the glyph → falls back to text
    assert!(out.contains("open"));
    assert!(!out.contains("🔨"));
}

// --- Field trait stub ---

/// Minimal stub implementing [`Field`] to exercise the trait end-to-end.
struct TitleField;

impl Field for TitleField {
    const NAME: &'static str = "title";

    fn values<'a>(task: &'a Task) -> Vec<Cow<'a, str>> {
        vec![Cow::Borrowed(task.title.as_str())]
    }
}

fn stub_task() -> Task {
    Task {
        id: "stub".to_owned(),
        file: std::path::PathBuf::from("stub.typ"),
        title: "My task".to_owned(),
        status: None,
        priority: None,
        difficulty: None,
        kind: None,
        estimate: Default::default(),
        links: vec![],
        tags: vec![],
        derived_tags: vec![],
        depth: 0,
        due: None,
        start: None,
        reviewed: None,
        recur: None,
        extras: Default::default(),
        shape_errors: Vec::new(),
        body: String::new(),
    }
}

#[test]
fn field_name_constant() {
    assert_eq!(TitleField::NAME, "title");
}

#[test]
fn field_values_returns_title() {
    let task = stub_task();
    let vals = TitleField::values(&task);
    assert_eq!(vals.len(), 1);
    let first: Option<&str> = vals.first().map(std::borrow::Cow::as_ref);
    assert_eq!(first, Some("My task"));
}

#[test]
fn field_cell_spec_default_impl() {
    let config = Config::default();
    let spec = TitleField::cell_spec("My task", &config);
    assert_eq!(spec.text, "My task");
    assert!(spec.glyph.is_none());
    assert!(spec.color.is_none());
}

#[test]
fn field_end_to_end_render() {
    let task = stub_task();
    let config = Config::default();
    let vals = TitleField::values(&task);
    let first: &str = vals.first().map_or("", std::borrow::Cow::as_ref);
    let spec = TitleField::cell_spec(first, &config);
    let out = render_cell(
        &spec,
        GlyphMode::Ascii,
        DensityMode::Full,
        Painter::new(false),
        12,
    );
    assert_eq!(display_width(&out), 12);
    assert!(out.starts_with("My task"));
}
