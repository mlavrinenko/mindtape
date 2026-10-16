//! A listing's last byte: every `mt ls --format` ends in exactly the
//! terminator its format calls for, the empty listing included
//! (`truth/tasks/print-one-newline-at-the-end-of-an-mt-ls-csv-listing.typ`).
#![allow(clippy::unwrap_used)]

use crate::support;

fn stdout_of(filter: &str, format: &str) -> Vec<u8> {
    let dir = support::project_with_tasks(&[("a.typ", "Alpha"), ("b.typ", "Beta")]);
    support::mt(dir.path())
        .args(["ls", filter, "--format", format])
        .output()
        .unwrap()
        .stdout
}

#[test]
fn csv_ends_with_one_newline_after_the_last_record() {
    let full = stdout_of("", "csv");
    assert!(
        full.ends_with(b"\n") && !full.ends_with(b"\n\n"),
        "{full:?}"
    );
}

#[test]
fn an_empty_csv_listing_is_the_header_and_one_newline() {
    assert_eq!(stdout_of("title == \"zz\"", "csv"), b"id,title\n");
}

#[test]
fn json_and_path_end_with_one_newline_and_path0_with_one_nul() {
    for format in ["json", "path"] {
        let out = stdout_of("", format);
        assert!(out.ends_with(b"\n") && !out.ends_with(b"\n\n"), "{format}");
    }
    let out = stdout_of("", "path0");
    assert!(out.ends_with(b"\0") && !out.ends_with(b"\n"), "{out:?}");
}

#[test]
fn empty_path_listings_print_nothing() {
    for format in ["path", "path0"] {
        assert!(stdout_of("title == \"zz\"", format).is_empty(), "{format}");
    }
}

#[test]
fn the_table_keeps_one_blank_line_before_its_footer() {
    let out = String::from_utf8(stdout_of("", "table")).unwrap();
    let (rows, footer) = out.split_once("\n\n").unwrap();
    assert!(!rows.is_empty() && !footer.contains("\n\n"), "{out:?}");
    assert!(
        footer.ends_with('\n') && !footer.ends_with("\n\n"),
        "{out:?}"
    );
}
