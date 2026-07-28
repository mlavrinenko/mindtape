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
use super::super::link::Linker;
use super::super::width::display_width;
use super::{
    Column, apply_flex, apply_highlight, flex_columns, glyph_mode, render_grid, render_unpadded,
};

/// Every layout test renders with hyperlinks off; only the width-accounting
/// test below turns the linker on.
fn no_links() -> Linker {
    Linker::new(false)
}

/// The `flex_columns`/`apply_flex` fixture shared by four tests below: a
/// narrow non-flex `#` column and a wide flex `title` column, neither linked.
fn handle_and_title_columns() -> Vec<Column> {
    vec![
        Column {
            id: ColumnId::Handle,
            cells: vec![],
            width: 3,
            link_uris: None,
        },
        Column {
            id: ColumnId::Title,
            cells: vec![],
            width: 200,
            link_uris: None,
        },
    ]
}

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
            link_uris: None,
        },
        Column {
            id: ColumnId::Status,
            cells: vec![CellSpec {
                text: "wip".into(),
                glyph: None,
                color: None,
            }],
            width: 3,
            link_uris: None,
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
        link_uris: None,
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
    let mut columns = handle_and_title_columns();
    flex_columns(&mut columns, 80);
    // avail = 80 - (3 + 2*1) = 75; min(200, 75) = 75; max(75, 16) = 75
    assert_eq!(columns[1].width, 75);
}

#[test]
fn flex_columns_respects_title_minimum() {
    let mut columns = handle_and_title_columns();
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
            link_uris: None,
        },
        Column {
            id: ColumnId::Status,
            cells: vec![],
            width: 6,
            link_uris: None,
        },
    ];
    flex_columns(&mut columns, 80);
    // No flex column → widths unchanged
    assert_eq!(columns[0].width, 4);
    assert_eq!(columns[1].width, 6);
}

#[test]
fn apply_flex_skips_when_no_terminal() {
    let mut columns = handle_and_title_columns();
    let old = columns[1].width;
    apply_flex(&mut columns, None);
    assert_eq!(
        columns[1].width, old,
        "width must be unchanged when terminal_width is None"
    );
}

#[test]
fn apply_flex_trims_when_terminal_known() {
    let mut columns = handle_and_title_columns();
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
            link_uris: None,
        },
        Column {
            id: ColumnId::Title,
            cells: vec![],
            width: 10,
            link_uris: None,
        },
    ];
    let out = render_grid(&cols, GlyphMode::None, Painter::new(false), no_links());
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
            link_uris: None,
        },
        Column {
            id: ColumnId::Title,
            cells: vec![CellSpec {
                text: "Hi".into(),
                glyph: None,
                color: None,
            }],
            width: 10,
            link_uris: None,
        },
    ];
    let out = render_grid(&cols, GlyphMode::None, Painter::new(false), no_links());
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
            link_uris: None,
        },
        Column {
            id: ColumnId::Status,
            cells: vec![],
            width: 4,
            link_uris: None,
        },
        Column {
            id: ColumnId::Title,
            cells: vec![],
            width: 100,
            link_uris: None,
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
        link_uris: None,
    };
    let out = render_grid(&[col], GlyphMode::None, Painter::new(false), no_links());
    assert!(out.starts_with("TITLE"), "header must be first line");
    assert!(out.contains("My task"), "content must follow");
}

/// The width-accounting trap this task exists to pin: rendering with
/// hyperlinks on must lay out identically to rendering with them off, once
/// the OSC 8 escapes are stripped back out. `link_uris` sits on the `file`
/// column only, exactly mirroring how a real `--columns '#,title,file'` run
/// would carry it, with `title` as the flex column and `file` last (so the
/// unpadded-last-column path is exercised too).
#[test]
fn hyperlink_wrap_does_not_disturb_column_layout() {
    let uri = "file:///abs/truth/tasks/x.typ";
    let build = |linked: bool| {
        vec![
            Column {
                id: ColumnId::Title,
                cells: vec![CellSpec {
                    text: "Rewrite auth".into(),
                    glyph: None,
                    color: None,
                }],
                width: 20,
                link_uris: None,
            },
            Column {
                id: ColumnId::File,
                cells: vec![CellSpec {
                    text: "truth/tasks/x.typ".into(),
                    glyph: None,
                    color: None,
                }],
                width: 20,
                link_uris: linked.then(|| vec![uri.to_owned()]),
            },
        ]
    };
    let with_links = render_grid(
        &build(true),
        GlyphMode::None,
        Painter::new(false),
        Linker::new(true),
    );
    let without_links = render_grid(
        &build(false),
        GlyphMode::None,
        Painter::new(false),
        no_links(),
    );

    assert_ne!(
        with_links, without_links,
        "sanity: turning hyperlinks on must actually add bytes"
    );

    let open = format!("\x1b]8;;{uri}\x1b\\");
    let close = "\x1b]8;;\x1b\\";
    let stripped = with_links.replacen(&open, "", 1).replacen(close, "", 1);
    assert_eq!(
        stripped, without_links,
        "stripping the OSC 8 wrap must restore an identical, unlinked layout \
         (same column positions and padding)"
    );
}

#[test]
fn only_the_file_column_is_wrapped_even_when_others_have_no_uris() {
    // A non-file column never carries `link_uris`, so even with hyperlinks
    // forced on, its cell must render byte-identical to hyperlinks off.
    let cols = vec![Column {
        id: ColumnId::Handle,
        cells: vec![CellSpec {
            text: "1a".into(),
            glyph: None,
            color: None,
        }],
        width: 2,
        link_uris: None,
    }];
    let linked = render_grid(
        &cols,
        GlyphMode::None,
        Painter::new(false),
        Linker::new(true),
    );
    assert!(
        !linked.contains('\u{1b}'),
        "a column with no link_uris must never be wrapped: {linked:?}"
    );
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
