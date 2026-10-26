#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use super::{Sources, key_line};
use crate::config::ConfigError;
use crate::config::pack_registry::PackOrigin;

const TEXT: &str = "extends = [\"x\"]\n\n[views.broken]\nsort = [\"a\"]\n\n[[on.flip]]\nwhen = \"w\"\n\n[[on.flip]]\nactions = [{ run = \"true\" }]\n";

#[test]
fn a_key_resolves_to_its_own_line() {
    assert_eq!(key_line(TEXT, "extends"), Some(1));
    assert_eq!(key_line(TEXT, "views.broken.sort"), Some(4));
    assert_eq!(key_line(TEXT, "views.broken"), Some(3));
}

#[test]
fn an_array_index_resolves_to_its_entry() {
    assert_eq!(key_line(TEXT, "on.flip[1]"), Some(9));
    assert_eq!(key_line(TEXT, "on.flip[1].actions"), Some(10));
    assert_eq!(key_line(TEXT, "on.flip[1].actions[0]"), Some(10));
}

#[test]
fn an_undeclared_tail_has_no_line() {
    assert_eq!(key_line(TEXT, "views.broken.filter"), None);
    assert_eq!(key_line(TEXT, "on.flip[2]"), None);
    assert_eq!(key_line(TEXT, "nothing"), None);
}

fn embedded(name: &str, text: &str) -> Sources {
    let mut sources = Sources::default();
    sources.push_pack(
        &PackOrigin::Embedded(name.to_owned()),
        text,
        Path::new("/p"),
    );
    sources
}

#[test]
fn a_key_no_source_declares_in_full_gets_no_place() {
    let sources = embedded("mindtape:x", "[views.board]\ntitle = \"a\"\n");
    assert_eq!(sources.place_of("views.board.filter: bad"), None);
    assert_eq!(sources.place_of("not a key: bad"), None);
}

#[test]
fn an_embedded_pack_is_named_by_its_source_and_pack() {
    let sources = embedded("mindtape:x", "[views.board]\nfilter = \"((\"\n");
    let place = sources.place_of("views.board.filter: bad").unwrap();
    assert_eq!(place.file, PathBuf::from("pack mindtape:x"));
    assert_eq!(place.line, 2);
}

#[test]
fn a_pack_beside_a_layer_is_searched_in_merge_precedence() {
    let mut sources = embedded("mindtape:x", "[views.board]\nfilter = \"a\"\n");
    sources.push_pack(
        &PackOrigin::Path("/p/team.toml".into()),
        "[views.board]\n\nfilter = \"b\"\n",
        Path::new("/p"),
    );
    let place = sources.place_of("views.board.filter: bad").unwrap();
    assert_eq!(place.file, PathBuf::from("team.toml"));
    assert_eq!(place.line, 3);
}

#[test]
fn an_embedded_pack_with_an_unknown_key_is_named() {
    let sources = embedded("mindtape:x", "[projet]\nx = 1\n");
    let merged = toml::from_str::<crate::config::Config>("[projet]\nx = 1\n").unwrap_err();
    let ConfigError::ParseIn { origin, .. } = sources.blame(merged) else {
        panic!("expected the pack to be named");
    };
    assert_eq!(origin, "pack mindtape:x");
}

#[test]
fn a_failure_no_file_shows_alone_keeps_the_merged_error() {
    let sources = embedded("mindtape:x", "[ls]\n");
    let merged = toml::from_str::<crate::config::Config>("[projet]\n").unwrap_err();
    assert!(matches!(sources.blame(merged), ConfigError::Parse(_)));
}
