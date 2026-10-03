#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

//! Hyperlink-wrap tests for `table.rs`: width accounting, which columns get
//! wrapped, the `slug#`/`file` shared-target invariant
//! (`truth/tasks/hyperlink-the-handle-column.typ`), and highlight+link
//! composition for the interactive picker. Split out of
//! `table_tests_layout.rs` to keep both files clear of the size gate.

use anstyle::{Effects, Style};
use mindtape::Task;
use mindtape::test_support::bare_task;

use super::super::TestCtx;
use super::super::color::Painter;
use super::super::columns::ColumnId;
use super::super::field::CellSpec;
use super::super::glyph::GlyphMode;
use super::super::link::Linker;
use super::{Column, apply_highlight, build_columns, render_grid};

/// Every test here renders with hyperlinks off unless it says otherwise.
fn no_links() -> Linker {
    Linker::new(false)
}

fn plain_cell(text: &str) -> CellSpec {
    CellSpec {
        text: text.to_owned(),
        glyph: None,
        color: None,
    }
}

// --- build_columns: the shared per-row target -------------------------------

/// The task this exists to pin: `slug#` and `file` must carry the identical
/// per-row target for a given row, because it is the same file. Built once
/// (`row_hyperlink_targets` called a single time in `build_columns`) and
/// cloned into both columns rather than recomputed per column, so there is no
/// way for the two to diverge.
#[test]
fn slug_and_file_columns_share_the_same_row_target() {
    let task = bare_task("x");
    let fixture = TestCtx::default();
    let rows: Vec<(String, &Task)> = vec![("a1b".to_owned(), &task)];
    let visible = [ColumnId::StemHash, ColumnId::Title, ColumnId::File];
    let columns = build_columns(&visible, &rows, fixture.ctx(), GlyphMode::None);

    let slug_uris = columns[0]
        .link_uris
        .clone()
        .expect("slug# column must carry link_uris");
    let title_uris = &columns[1].link_uris;
    let file_uris = columns[2]
        .link_uris
        .clone()
        .expect("file column must carry link_uris");

    assert!(title_uris.is_none(), "title must never carry link_uris");
    assert_eq!(
        slug_uris, file_uris,
        "# and file must target the same file for a given row"
    );
}

/// A column with no `link_uris` (every column but `slug#`/`file`) is untouched by
/// hyperlinks even when the linker is forced on.
#[test]
fn columns_without_link_uris_are_never_wrapped() {
    let cols = vec![Column {
        id: ColumnId::Status,
        cells: vec![plain_cell("wip")],
        width: 3,
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

// --- render_grid: width accounting ------------------------------------------

/// The width-accounting trap this task exists to pin: rendering with
/// hyperlinks on must lay out identically to rendering with them off, once
/// the OSC 8 escapes are stripped back out. Both `slug#` and `file` carry
/// `link_uris` here, exactly mirroring a real `--columns slug#,title,file` run,
/// with `title` as the flex column and `file` last (so the
/// unpadded-last-column path is exercised too).
#[test]
fn hyperlink_wrap_does_not_disturb_column_layout() {
    let uri = "file:///abs/truth/tasks/x.typ";
    let build = |linked: bool| {
        vec![
            Column {
                id: ColumnId::StemHash,
                cells: vec![plain_cell("1a")],
                width: 2,
                link_uris: linked.then(|| vec![uri.to_owned()]),
            },
            Column {
                id: ColumnId::Title,
                cells: vec![plain_cell("Rewrite auth")],
                width: 20,
                link_uris: None,
            },
            Column {
                id: ColumnId::File,
                cells: vec![plain_cell("truth/tasks/x.typ")],
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
    // Two linked columns now, so both opens/closes must be stripped.
    let stripped = with_links.replace(&open, "").replace(close, "");
    assert_eq!(
        stripped, without_links,
        "stripping the OSC 8 wrap from both # and file must restore an \
         identical, unlinked layout (same column positions and padding)"
    );
}

// --- highlight + link composition (the interactive risk) -------------------

/// The real risk this task exists to guard against: `apply_highlight` colours
/// the slug# cell reverse-video for `mt ls -i`, and the same cell now also
/// carries a hyperlink target. The two must compose — the invert style rides
/// inside the OSC 8 wrap without disturbing it, and stripping the OSC 8 wrap
/// back out restores exactly the highlighted (but unlinked) rendering.
#[test]
fn highlighted_handle_cell_still_links_and_keeps_its_shape() {
    let uri = "file:///abs/truth/tasks/x.typ";
    let build = |linked: bool| {
        let mut columns = vec![Column {
            id: ColumnId::StemHash,
            cells: vec![plain_cell("1a")],
            width: 4,
            link_uris: linked.then(|| vec![uri.to_owned()]),
        }];
        apply_highlight(&mut columns, &[true]);
        columns
    };

    let with_link = render_grid(
        &build(true),
        GlyphMode::None,
        Painter::new(true),
        Linker::new(true),
    );
    let without_link = render_grid(
        &build(false),
        GlyphMode::None,
        Painter::new(true),
        no_links(),
    );

    let invert = Style::new().effects(Effects::INVERT).to_string();
    let data_line = with_link.lines().nth(1).expect("must have a data row");
    assert!(
        data_line.contains(&invert),
        "the invert style must still apply under the hyperlink wrap: {data_line:?}"
    );

    let open = format!("\x1b]8;;{uri}\x1b\\");
    let close = "\x1b]8;;\x1b\\";
    let stripped = with_link.replacen(&open, "", 1).replacen(close, "", 1);
    assert_eq!(
        stripped, without_link,
        "stripping the OSC 8 wrap must restore the highlighted-but-unlinked \
         rendering exactly, proving link and highlight compose"
    );
}
