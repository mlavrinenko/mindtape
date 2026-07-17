#![allow(clippy::unwrap_used)]

use super::{Anchor, Band, NumValue, NumericField, ScaleKind};
use crate::config::Requirement;

fn band(label: &str, min: f64, max: f64) -> Band {
    Band {
        label: label.to_owned(),
        min,
        max,
        glyph: Some(format!("[{label}]")),
        color: Some("green".to_owned()),
    }
}

fn anchor(value: f64, label: &str) -> Anchor {
    Anchor {
        value,
        label: label.to_owned(),
        glyph: Some(format!("[{label}]")),
        color: Some("blue".to_owned()),
    }
}

fn int_bands() -> NumericField {
    NumericField {
        kind: ScaleKind::Int,
        min: 0.0,
        max: 9.0,
        bands: vec![
            band("low", 0.0, 2.0),
            band("normal", 3.0, 6.0),
            band("high", 7.0, 9.0),
        ],
        anchors: Vec::new(),
        required: Requirement::No,
    }
}

fn anchors() -> NumericField {
    NumericField {
        kind: ScaleKind::Int,
        min: 0.0,
        max: 9.0,
        bands: Vec::new(),
        anchors: vec![
            anchor(0.0, "junior"),
            anchor(3.0, "mid"),
            anchor(6.0, "senior"),
            anchor(9.0, "staff"),
        ],
        required: Requirement::No,
    }
}

#[test]
fn num_value_parse_and_display() {
    assert_eq!(NumValue::parse("4"), NumValue::Num(4.0));
    assert_eq!(NumValue::parse("high"), NumValue::Label("high".to_owned()));
    assert_eq!(NumValue::Num(8.0).as_display(), "8");
    assert_eq!(NumValue::Num(2.5).as_display(), "2.5");
    assert_eq!(NumValue::Label("x".to_owned()).as_display(), "x");
}

#[test]
fn band_classify_picks_the_containing_band() {
    let field = int_bands();
    assert_eq!(field.classify(2.0).unwrap().label, "low");
    assert_eq!(field.classify(5.0).unwrap().label, "normal");
    assert_eq!(field.classify(9.0).unwrap().label, "high");
    assert_eq!(field.classify(0.0).unwrap().label, "low");
}

#[test]
fn band_label_resolves_to_representative_value() {
    let field = int_bands();
    assert_eq!(
        field.resolve(&NumValue::Label("high".to_owned())),
        Some(7.0)
    );
    assert_eq!(field.resolve(&NumValue::Num(4.0)), Some(4.0));
    assert_eq!(field.resolve(&NumValue::Label("nope".to_owned())), None);
}

#[test]
fn real_bands_close_only_the_last() {
    let field = NumericField {
        kind: ScaleKind::Real,
        min: 0.0,
        max: 1.0,
        bands: vec![band("lo", 0.0, 0.5), band("hi", 0.5, 1.0)],
        anchors: Vec::new(),
        required: Requirement::No,
    };
    assert_eq!(field.classify(0.5).unwrap().label, "hi");
    assert_eq!(field.classify(0.25).unwrap().label, "lo");
    assert_eq!(field.classify(1.0).unwrap().label, "hi");
}

#[test]
fn anchor_exact_and_derived_labels() {
    let field = anchors();
    let cell = field.cell(&NumValue::Label("senior".to_owned())).unwrap();
    assert_eq!(cell.label, "senior");
    let derived = field.classify(7.0).unwrap();
    assert_eq!(derived.label, "senior+");
    assert_eq!(derived.glyph.as_deref(), Some("[senior]"));
    assert_eq!(field.classify(8.0).unwrap().label, "senior++");
}

#[test]
fn anchor_above_top_derives_from_highest() {
    let field = anchors();
    // 9 is the staff anchor exactly.
    assert_eq!(field.classify(9.0).unwrap().label, "staff");
}

#[test]
fn valid_configs_report_no_problems() {
    assert!(int_bands().validate("priority").is_empty());
    assert!(anchors().validate("difficulty").is_empty());
}

#[test]
fn bands_with_a_gap_are_rejected() {
    let mut field = int_bands();
    field.bands = vec![band("low", 0.0, 2.0), band("high", 5.0, 9.0)];
    let problems = field.validate("priority");
    assert!(
        problems.iter().any(|m| m.contains("contiguous")),
        "{problems:?}"
    );
}

#[test]
fn real_scale_with_floor_shaped_int_bands_hints_at_the_fix() {
    let mut field = int_bands();
    field.kind = ScaleKind::Real;
    let problems = field.validate("priority");
    assert!(
        problems
            .iter()
            .any(|m| m.contains("not contiguous") && m.contains("bands = []")),
        "{problems:?}"
    );
}

#[test]
fn real_scale_with_a_genuine_gap_has_no_floor_hint() {
    let mut field = int_bands();
    field.kind = ScaleKind::Real;
    field.bands = vec![band("low", 0.0, 1.0), band("high", 5.0, 9.0)];
    let problems = field.validate("priority");
    assert!(
        problems.iter().any(|m| m.contains("not contiguous")),
        "{problems:?}"
    );
    assert!(
        problems.iter().all(|m| !m.contains("bands = []")),
        "{problems:?}"
    );
}

#[test]
fn int_scale_with_floor_shaped_bands_is_still_valid() {
    assert!(int_bands().validate("priority").is_empty());
}

#[test]
fn bands_not_covering_scale_are_rejected() {
    let mut field = int_bands();
    field.bands = vec![band("low", 0.0, 2.0), band("normal", 3.0, 6.0)];
    let problems = field.validate("priority");
    assert!(
        problems.iter().any(|m| m.contains("end at 9")),
        "{problems:?}"
    );
}

#[test]
fn mixing_bands_and_anchors_is_rejected() {
    let mut field = int_bands();
    field.anchors = vec![anchor(0.0, "junior")];
    let problems = field.validate("priority");
    assert!(
        problems.iter().any(|m| m.contains("not both")),
        "{problems:?}"
    );
}

#[test]
fn anchors_without_one_at_min_are_rejected() {
    let mut field = anchors();
    field.anchors = vec![anchor(3.0, "mid"), anchor(6.0, "senior")];
    let problems = field.validate("difficulty");
    assert!(
        problems.iter().any(|m| m.contains("anchor at 0")),
        "{problems:?}"
    );
}

#[test]
fn band_with_min_above_max_is_rejected() {
    let mut field = int_bands();
    // A single band spanning the whole scale but with min/max inverted.
    field.bands = vec![band("backwards", 9.0, 0.0)];
    let problems = field.validate("priority");
    assert!(
        problems.iter().any(|m| m.contains("min above max")),
        "{problems:?}"
    );
}

#[test]
fn single_point_band_is_not_flagged_as_inverted() {
    let mut field = int_bands();
    // A degenerate but legal band where min == max: `>` tolerates it, so the
    // "min above max" message must not fire (pins the boundary against `>=`).
    field.bands = vec![band("point", 5.0, 5.0)];
    let problems = field.validate("priority");
    assert!(
        problems.iter().all(|m| !m.contains("min above max")),
        "{problems:?}"
    );
}

#[test]
fn anchor_out_of_range_is_rejected() {
    let mut field = anchors();
    // Keep the required min anchor, add one above the scale max (9).
    field.anchors = vec![anchor(0.0, "junior"), anchor(12.0, "beyond")];
    let problems = field.validate("difficulty");
    assert!(
        problems
            .iter()
            .any(|m| m.contains("beyond") && m.contains("out of range")),
        "{problems:?}"
    );
}

#[test]
fn point_below_lowest_anchor_derives_downward() {
    let mut field = anchors();
    // Lowest anchor at 3 so an in-scale point (1) sits below every anchor and
    // derives downward with the U+2212 minus, one per step of distance.
    field.anchors = vec![anchor(3.0, "mid"), anchor(6.0, "senior")];
    let cell = field.classify(1.0).unwrap();
    assert_eq!(cell.label, "mid\u{2212}\u{2212}");
    assert_eq!(cell.glyph.as_deref(), Some("[mid]"));
}
