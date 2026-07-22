#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

//! Layout-side `table` tests: highlight masking, flex width distribution,
//! grid rendering and unpadded-cell truncation. Split out of `table_tests.rs`
//! to keep both files under the size gate; sibling of `table_tests.rs`
//! (render/due-cell) and `table_tests_ready.rs` (`ready_cell` floor).

use std::io::IsTerminal;

use anstyle::{Effects, Style};

use super::super::color::Painter;
use super::super::columns::ColumnId;
use super::super::field::CellSpec;
use super::super::glyph::GlyphMode;
use super::super::width::display_width;
use super::{
    Column, apply_flex, apply_highlight, flex_columns, glyph_mode, render_grid, render_unpadded,
};

// --- apply_highlight --------------------------------------------------------

#[test]
fn apply_highlight_inverts_handle_cell() {
    let spec = CellSpec {
        text: "1a".into(),
        glyph: None,
        color: None,
    };
    let mut columns = vec![
        Column {
            id: ColumnId::Handle,
            cells: vec![spec.clone()],
            width: 2,
        },
        Column {
            id: ColumnId::Status,
            cells: vec![CellSpec {
                text: "wip".into(),
                glyph: None,
                color: None,
            }],
            width: 3,
        },
    ];
    apply_highlight(&mut columns, &[true]);
    assert!(
        columns[0].cells[0].color.is_some(),
        "handle cell must be highlighted"
    );
    assert!(
        columns[1].cells[0].color.is_none(),
        "status cell must NOT be highlighted"
    );
}

#[test]
fn apply_highlight_skips_when_none_flagged() {
    let invert = Style::new().effects(Effects::INVERT);
    let spec = CellSpec {
        text: "1a".into(),
        glyph: None,
        color: Some(invert),
    };
    let mut columns = vec![Column {
        id: ColumnId::Handle,
        cells: vec![spec],
        width: 2,
    }];
    apply_highlight(&mut columns, &[false]);
    assert!(
        columns[0].cells[0].color.is_some(),
        "no-flag must preserve existing color"
    );
}

// --- flex_columns -----------------------------------------------------------

#[test]
fn flex_columns_trims_title_to_fit_terminal() {
    let mut columns = vec![
        Column {
            id: ColumnId::Handle,
            cells: vec![],
            width: 3,
        },
        Column {
            id: ColumnId::Title,
            cells: vec![],
            width: 200,
        },
    ];
    flex_columns(&mut columns, 80);
    // avail = 80 - (3 + 2*1) = 75; min(200, 75) = 75; max(75, 16) = 75
    assert_eq!(columns[1].width, 75);
}

#[test]
fn flex_columns_respects_title_minimum() {
    let mut columns = vec![
        Column {
            id: ColumnId::Handle,
            cells: vec![],
            width: 3,
        },
        Column {
            id: ColumnId::Title,
            cells: vec![],
            width: 200,
        },
    ];
    flex_columns(&mut columns, 0);
    // avail = 0.saturating_sub(5) = 0; min(200, 0) = 0; max(0, 16) = 16
    assert_eq!(columns[1].width, 16);
}

#[test]
fn flex_columns_handles_non_flex_only() {
    let mut columns = vec![
        Column {
            id: ColumnId::Handle,
            cells: vec![],
            width: 4,
        },
        Column {
            id: ColumnId::Status,
            cells: vec![],
            width: 6,
        },
    ];
    flex_columns(&mut columns, 80);
    // No flex column → widths unchanged
    assert_eq!(columns[0].width, 4);
    assert_eq!(columns[1].width, 6);
}

#[test]
fn apply_flex_skips_when_no_terminal() {
    let mut columns = vec![
        Column {
            id: ColumnId::Handle,
            cells: vec![],
            width: 3,
        },
        Column {
            id: ColumnId::Title,
            cells: vec![],
            width: 200,
        },
    ];
    let old = columns[1].width;
    apply_flex(&mut columns, None);
    assert_eq!(
        columns[1].width, old,
        "width must be unchanged when terminal_width is None"
    );
}

#[test]
fn apply_flex_trims_when_terminal_known() {
    let mut columns = vec![
        Column {
            id: ColumnId::Handle,
            cells: vec![],
            width: 3,
        },
        Column {
            id: ColumnId::Title,
            cells: vec![],
            width: 200,
        },
    ];
    apply_flex(&mut columns, Some(80));
    assert!(
        columns[1].width < 200,
        "flex column must shrink when terminal_width is known"
    );
}

#[test]
fn render_grid_header_last_column_not_padded() {
    let cols = vec![
        Column {
            id: ColumnId::Handle,
            cells: vec![],
            width: 2,
        },
        Column {
            id: ColumnId::Title,
            cells: vec![],
            width: 10,
        },
    ];
    let out = render_grid(&cols, GlyphMode::None, Painter::new(false));
    let header = out.lines().next().unwrap();
    assert!(
        !header.ends_with(' '),
        "last column header must not pad: {header:?}"
    );
}

#[test]
fn render_grid_last_column_not_padded() {
    let cols = vec![
        Column {
            id: ColumnId::Handle,
            cells: vec![CellSpec {
                text: "1a".into(),
                glyph: None,
                color: None,
            }],
            width: 2,
        },
        Column {
            id: ColumnId::Title,
            cells: vec![CellSpec {
                text: "Hi".into(),
                glyph: None,
                color: None,
            }],
            width: 10,
        },
    ];
    let out = render_grid(&cols, GlyphMode::None, Painter::new(false));
    let data_line: &str = out.lines().nth(1).unwrap();
    assert!(
        !data_line.ends_with(' '),
        "last column must not pad: {data_line:?}"
    );
}

#[test]
fn flex_columns_uses_multiplication_for_gaps() {
    // Three columns so gaps > 1 (exposes * vs / in GAP.len() * gaps)
    let mut cols = vec![
        Column {
            id: ColumnId::Handle,
            cells: vec![],
            width: 2,
        },
        Column {
            id: ColumnId::Status,
            cells: vec![],
            width: 4,
        },
        Column {
            id: ColumnId::Title,
            cells: vec![],
            width: 100,
        },
    ];
    flex_columns(&mut cols, 50);
    // gaps = 2, GAP.len() = 2 → gap_cost = 4 (not 1, which / would give)
    // others = 2 + 4 = 6, avail = 50 - (6 + 4) = 40
    assert_eq!(cols[2].width, 40);
}

// --- render_grid -----------------------------------------------------------

#[test]
fn render_grid_single_column_shows_header_then_content() {
    let col = Column {
        id: ColumnId::Title,
        cells: vec![CellSpec {
            text: "My task".into(),
            glyph: None,
            color: None,
        }],
        width: 7,
    };
    let out = render_grid(&[col], GlyphMode::None, Painter::new(false));
    assert!(out.starts_with("TITLE"), "header must be first line");
    assert!(out.contains("My task"), "content must follow");
}

// --- render_unpadded -------------------------------------------------------

#[test]
fn render_unpadded_truncates_when_over_width() {
    let spec = CellSpec {
        text: "very long text".into(),
        glyph: None,
        color: None,
    };
    let out = render_unpadded(&spec, GlyphMode::None, Painter::new(false), 5);
    assert_eq!(display_width(&out), 5);
    assert!(out.ends_with('…'), "truncated must end with ellipsis");
}

#[test]
fn render_unpadded_returns_content_as_is_when_fits() {
    let spec = CellSpec {
        text: "hi".into(),
        glyph: None,
        color: None,
    };
    let out = render_unpadded(&spec, GlyphMode::None, Painter::new(false), 10);
    assert_eq!(out, "hi", "must not add trailing padding");
}

// --- glyph_mode (non-TTY) --------------------------------------------------

#[test]
fn glyph_mode_follows_whether_stdout_is_a_terminal() {
    // `glyph_mode` has exactly one input, and it is not something this test can
    // set: asserting a bare `GlyphMode::None` stated a property of the harness
    // (cargo pipes stdout), not of the code, and would flip meaning the moment
    // the suite ran on a tty. Assert the rule against the same input instead.
    let expected = if std::io::stdout().is_terminal() {
        GlyphMode::Emoji
    } else {
        GlyphMode::None
    };
    assert_eq!(glyph_mode(), expected);
}
