#![allow(clippy::unwrap_used)]

use super::{has_kind, sugar_kind};
use crate::link::Link;
use crate::test_support::bare_task;

fn edge(target: &str, kind: &str) -> Link {
    Link {
        task: target.to_owned(),
        kind: kind.to_owned(),
        note: None,
    }
}

#[test]
fn sugar_kind_maps_legacy_names() {
    assert_eq!(sugar_kind("depends_on"), Some("depends-on"));
    assert_eq!(sugar_kind("supersedes"), Some("supersedes"));
    assert_eq!(sugar_kind("duplicates"), Some("duplicates"));
    assert_eq!(sugar_kind("relates_to"), None);
    assert_eq!(sugar_kind("related"), None);
}

#[test]
fn has_kind_checks_presence() {
    let mut task = bare_task("a");
    task.links = vec![edge("b.typ", "depends-on")];
    assert!(has_kind(&task, "depends-on"));
    assert!(!has_kind(&task, "related"));
}
