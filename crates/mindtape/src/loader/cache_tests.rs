#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::test_support::tempdir;
use std::collections::BTreeMap;

use super::{
    Cached, CachedOutcome, TaskCache, build_key, disabled, ensure_ignored, is_build_key,
    json_to_hval, prune_stale,
};
use crate::config::Config;
use crate::model::canonical::hval_to_json;
use typst_harvest::HVal;

fn open(root: &std::path::Path) -> TaskCache {
    TaskCache::open(root, &Config::default(), &[])
}

#[test]
fn content_hash_is_stable_and_content_sensitive() {
    assert_eq!(
        TaskCache::content_hash(b"same bytes"),
        TaskCache::content_hash(b"same bytes")
    );
    assert_ne!(
        TaskCache::content_hash(b"one"),
        TaskCache::content_hash(b"two")
    );
    // 40-char lowercase SHA-1 hex.
    let hash = TaskCache::content_hash(b"x");
    assert_eq!(hash.len(), 40);
    assert!(hash.bytes().all(|b| b.is_ascii_hexdigit()));
}

#[test]
fn put_get_roundtrips_a_marker() {
    let root = tempdir();
    let cache = open(root.path());
    let marker = HVal::Dict(BTreeMap::from([
        ("marker".to_owned(), HVal::Str("mindtape.task".to_owned())),
        (
            "fields".to_owned(),
            HVal::Dict(BTreeMap::from([
                ("title".to_owned(), HVal::Str("T".to_owned())),
                ("priority".to_owned(), HVal::Int(8)),
                ("ratio".to_owned(), HVal::Float(0.5)),
                ("flag".to_owned(), HVal::Bool(true)),
                (
                    "tags".to_owned(),
                    HVal::Array(vec![HVal::Str("a".to_owned())]),
                ),
            ])),
        ),
        ("extras".to_owned(), HVal::Dict(BTreeMap::new())),
    ]));

    let hash = TaskCache::content_hash(b"source");
    cache.put(&hash, Some(&marker), &[], &[]);

    match cache.get(&hash) {
        Some(Cached {
            outcome: CachedOutcome::Marker(got),
            ..
        }) => assert_eq!(got, marker),
        _ => panic!("expected a marker hit"),
    }
}

#[test]
fn nomarker_outcome_roundtrips() {
    let root = tempdir();
    let cache = open(root.path());
    let hash = TaskCache::content_hash(b"no-marker");
    cache.put(&hash, None, &[], &[]);
    assert!(matches!(
        cache.get(&hash),
        Some(Cached {
            outcome: CachedOutcome::NoMarker,
            ..
        })
    ));
}

#[test]
fn missing_entry_is_a_miss() {
    let root = tempdir();
    let cache = open(root.path());
    assert!(
        cache
            .get(&TaskCache::content_hash(b"never-written"))
            .is_none()
    );
}

#[test]
fn corrupt_or_wrong_format_entry_is_a_miss() {
    let root = tempdir();
    let cache = open(root.path());
    let hash = TaskCache::content_hash(b"entry");
    cache.put(&hash, None, &[], &[]);
    let path = cache.entry_path(&hash).unwrap();

    std::fs::write(&path, "{ not json").unwrap();
    assert!(cache.get(&hash).is_none(), "unparseable entry must miss");

    std::fs::write(&path, r#"{"format":999,"marker":null}"#).unwrap();
    assert!(cache.get(&hash).is_none(), "wrong-format entry must miss");
}

#[test]
fn build_key_reacts_to_config_and_prelude() {
    let base = Config::default();
    let mut tweaked = Config::default();
    tweaked.project.code = Some("other".to_owned());
    assert_ne!(
        build_key(&base, &[]),
        build_key(&tweaked, &[]),
        "a config change must bust the key"
    );

    let lib = tempdir();
    std::fs::write(lib.path().join("prelude.typ"), "a").unwrap();
    let overrides = vec![("mindtape".to_owned(), lib.path().to_path_buf())];
    let before = build_key(&base, &overrides);
    std::fs::write(lib.path().join("prelude.typ"), "b").unwrap();
    let after = build_key(&base, &overrides);
    assert_ne!(before, after, "a prelude edit must bust the key");
}

#[test]
fn prune_stale_removes_other_build_keys_only() {
    let root = tempdir();
    let cache_root = root.path().join("cache");
    let keep = "a".repeat(40);
    let stale = "b".repeat(40);
    std::fs::create_dir_all(cache_root.join(&keep)).unwrap();
    std::fs::create_dir_all(cache_root.join(&stale)).unwrap();
    std::fs::write(cache_root.join(".gitignore"), "*\n").unwrap();

    prune_stale(&cache_root, &keep);

    assert!(cache_root.join(&keep).exists(), "current key kept");
    assert!(!cache_root.join(&stale).exists(), "stale key removed");
    assert!(
        cache_root.join(".gitignore").exists(),
        "non-key file untouched"
    );
}

#[test]
fn is_build_key_matches_sha1_hex_only() {
    assert!(is_build_key(&"a".repeat(40)));
    assert!(!is_build_key("short"));
    assert!(!is_build_key(&"a".repeat(41)));
    assert!(!is_build_key(&"g".repeat(40)));
}

#[test]
fn ensure_ignored_writes_self_ignoring_gitignore() {
    let root = tempdir();
    let cache_root = root.path().join("cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    ensure_ignored(&cache_root);
    assert_eq!(
        std::fs::read_to_string(cache_root.join(".gitignore")).unwrap(),
        "*\n"
    );
}

#[test]
fn json_hval_roundtrip_preserves_scalars_and_nesting() {
    let value = HVal::Dict(BTreeMap::from([
        ("int".to_owned(), HVal::Int(-3)),
        ("float".to_owned(), HVal::Float(1.5)),
        ("null".to_owned(), HVal::Null),
        (
            "arr".to_owned(),
            HVal::Array(vec![
                HVal::Bool(false),
                HVal::Int(0),
                HVal::Str("s".to_owned()),
            ]),
        ),
    ]));
    assert_eq!(json_to_hval(&hval_to_json(&value)), value);
}

#[test]
fn json_hval_roundtrip_collapses_date_to_string() {
    // JSON cannot carry the date/string distinction; a date reads back as a
    // string. Harmless — consumers read both via `HVal::as_str`.
    let dated = HVal::Date("2026-01-02".to_owned());
    assert_eq!(
        json_to_hval(&hval_to_json(&dated)),
        HVal::Str("2026-01-02".to_owned())
    );
}

#[test]
fn disabled_checks_env_var_semantics() {
    // Self-dual pattern: children spawned below run the env-var assertions.
    if std::env::var_os("MINDTAPE_DISABLED_CHILD").is_some() {
        let val = std::env::var_os("MINDTAPE_NO_CACHE");
        match val.as_deref().and_then(|v| v.to_str()) {
            Some("1") => assert!(disabled()),
            Some("") => assert!(!disabled()),
            _ => panic!("unexpected MINDTAPE_NO_CACHE value: {val:?}"),
        }
        return;
    }

    let binary = std::env::current_exe().unwrap();

    let out = std::process::Command::new(&binary)
        .env("MINDTAPE_NO_CACHE", "1")
        .env("MINDTAPE_DISABLED_CHILD", "1")
        .arg("disabled_checks_env_var_semantics")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "nonempty child:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    let out = std::process::Command::new(&binary)
        .env("MINDTAPE_NO_CACHE", "")
        .env("MINDTAPE_DISABLED_CHILD", "1")
        .arg("disabled_checks_env_var_semantics")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "empty child:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}
