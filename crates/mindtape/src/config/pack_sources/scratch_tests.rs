#![allow(clippy::unwrap_used)]

use std::time::Duration;

use super::{STALE_AFTER, sweep};
use crate::test_support::{tempdir, write_file};

#[test]
fn sweep_removes_only_scratch_directories_past_their_age() {
    let cache = tempdir();
    write_file(cache.path(), ".fetch-old/git/HEAD", "ref\n");
    write_file(cache.path(), "keep/a.toml", "a = 1\n");
    write_file(cache.path(), ".fetch-file", "not a directory\n");
    sweep(cache.path(), STALE_AFTER);
    assert!(
        cache.path().join(".fetch-old").is_dir(),
        "a fresh one stays"
    );
    sweep(cache.path(), Duration::ZERO);
    assert!(!cache.path().join(".fetch-old").exists());
    assert!(cache.path().join("keep/a.toml").is_file());
    assert!(cache.path().join(".fetch-file").is_file());
}
