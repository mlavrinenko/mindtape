#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::test_support::write_file;
use crate::test_support::{tempdir, tempdir_with_files};
use std::collections::BTreeMap;

use super::{
    Cached, CachedOutcome, TaskCache, build_key, disabled, ensure_ignored, is_build_key,
    prune_stale,
};
use crate::config::Config;
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
        build_key(&base, &[], None),
        build_key(&tweaked, &[], None),
        "a config change must bust the key"
    );

    let lib = tempdir_with_files(&[("prelude.typ", "a")]);
    let overrides = vec![("mindtape".to_owned(), lib.path().to_path_buf())];
    let before = build_key(&base, &overrides, None);
    write_file(lib.path(), "prelude.typ", "b");
    let after = build_key(&base, &overrides, None);
    assert_ne!(before, after, "a prelude edit must bust the key");
}

/// The key follows the directory an unoverridden `@local` import reads, so
/// moving it, as `$TYPST_PACKAGE_PATH` does, never replays a harvest made
/// against another prelude.
#[test]
fn build_key_reacts_to_the_package_root() {
    let base = Config::default();
    let one = std::path::Path::new("/one/typst/packages");
    let two = std::path::Path::new("/two/typst/packages");
    let here = build_key(&base, &[], Some(one));
    assert_ne!(here, build_key(&base, &[], Some(two)));
    assert_ne!(here, build_key(&base, &[], None));
    assert_eq!(here, build_key(&base, &[], Some(one)));
}

#[test]
fn prune_stale_removes_other_build_keys_only() {
    let root = tempdir();
    let cache_root = root.path().join("cache");
    let keep = "a".repeat(40);
    let stale = "b".repeat(40);
    std::fs::create_dir_all(cache_root.join(&keep)).unwrap();
    std::fs::create_dir_all(cache_root.join(&stale)).unwrap();
    write_file(&cache_root, ".gitignore", "*\n");

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

    for value in ["1", ""] {
        let out = crate::test_support::run_self_test(
            &crate::test_support::current_test_name(),
            "MINDTAPE_DISABLED_CHILD",
            |child| {
                child.env("MINDTAPE_NO_CACHE", value);
            },
        );
        crate::test_support::assert_child_ok(&out);
    }
}
