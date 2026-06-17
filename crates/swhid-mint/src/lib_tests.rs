#![allow(clippy::unwrap_used)]

use super::{Kind, Swhid, is_swhid_url};

const ZERO: &str = "0000000000000000000000000000000000000000";

#[test]
fn round_trips_rev_with_path() {
    let text = format!("swh:1:rev:{ZERO};path=truth/tasks/a.typ");
    let id = Swhid::parse(&text).expect("parse");
    assert_eq!(id.kind, Kind::Revision);
    assert_eq!(id.hash, ZERO);
    assert_eq!(id.path.as_deref(), Some("truth/tasks/a.typ"));
    assert_eq!(id.render(), text);
}

#[test]
fn round_trips_cnt_without_path() {
    let text = format!("swh:1:cnt:{ZERO}");
    let id = Swhid::parse(&text).expect("parse");
    assert_eq!(id.kind, Kind::Content);
    assert!(id.path.is_none());
    assert_eq!(id.render(), text);
}

#[test]
fn rejects_wrong_version() {
    assert!(Swhid::parse(&format!("swh:2:rev:{ZERO}")).is_none());
}

#[test]
fn rejects_unknown_kind() {
    assert!(Swhid::parse(&format!("swh:1:xyz:{ZERO}")).is_none());
}

#[test]
fn rejects_short_hash() {
    assert!(Swhid::parse("swh:1:rev:abc").is_none());
}

#[test]
fn rejects_uppercase_hash() {
    let upper = "A".repeat(40);
    assert!(Swhid::parse(&format!("swh:1:rev:{upper}")).is_none());
}

#[test]
fn ignores_unknown_qualifiers_keeps_path() {
    let text = format!("swh:1:rev:{ZERO};origin=x;path=p.typ;lines=1");
    let id = Swhid::parse(&text).expect("parse");
    assert_eq!(id.path.as_deref(), Some("p.typ"));
}

#[test]
fn recognizes_swhid_url() {
    assert!(is_swhid_url(&format!(
        "swh:1:rev:{ZERO};path=tasks/gone.typ"
    )));
    assert!(!is_swhid_url("file.typ"));
    assert!(!is_swhid_url("https://example.com/swh:1:rev:x"));
}
