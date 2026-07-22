//! `ready_cell` floor-default and override tests, split out of
//! `table_tests.rs` to keep it under the size gate.
//!
//! `Config::default()` bypasses `defaults.toml`; these regression tests go
//! through `parse_config` so they exercise the actual shipped floor, not
//! just `ready_cell`'s own Rust-side literal fallback.

use mindtape::{ReadyState, parse_config};

use super::super::color::parse_color;
use super::super::columns::ready_cell;

#[test]
fn ready_cell_floor_default_ready() {
    let config = parse_config("").expect("empty config parses against the embedded floor");
    let cell = ready_cell(ReadyState::Ready, &config);
    assert_eq!(cell.text, "ready");
    assert_eq!(cell.glyph.as_deref(), Some("▶"));
    assert_eq!(cell.color, parse_color("green"));
}

#[test]
fn ready_cell_floor_default_blocked() {
    let config = parse_config("").expect("empty config parses against the embedded floor");
    let cell = ready_cell(ReadyState::Blocked, &config);
    assert_eq!(cell.text, "blocked");
    assert_eq!(cell.glyph.as_deref(), Some("🔒"));
    assert_eq!(cell.color, parse_color("red"));
}

#[test]
fn ready_cell_settled_is_plain() {
    let config = parse_config("").expect("empty config parses against the embedded floor");
    let cell = ready_cell(ReadyState::Settled, &config);
    assert_eq!(cell.text, "");
    assert!(cell.glyph.is_none());
    assert!(cell.color.is_none());
}

#[test]
fn ready_cell_honors_ready_glyph_override() {
    let config =
        parse_config("[ls.ready]\nready-glyph = \">\"\n").expect("valid ls.ready override parses");
    let cell = ready_cell(ReadyState::Ready, &config);
    assert_eq!(cell.glyph.as_deref(), Some(">"));
}

#[test]
fn ready_cell_honors_ready_color_and_label_override() {
    let config = parse_config("[ls.ready]\nready-color = \"blue\"\nready-label = \"go\"\n")
        .expect("valid ls.ready override parses");
    let cell = ready_cell(ReadyState::Ready, &config);
    assert_eq!(cell.text, "go");
    assert_eq!(cell.color, parse_color("blue"));
}

#[test]
fn ready_cell_honors_blocked_overrides() {
    let config = parse_config(
        "[ls.ready]\nblocked-glyph = \"x\"\nblocked-color = \"magenta\"\nblocked-label = \"stuck\"\n",
    )
    .expect("valid ls.ready override parses");
    let cell = ready_cell(ReadyState::Blocked, &config);
    assert_eq!(cell.text, "stuck");
    assert_eq!(cell.glyph.as_deref(), Some("x"));
    assert_eq!(cell.color, parse_color("magenta"));
}
