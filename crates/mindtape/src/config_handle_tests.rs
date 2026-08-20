//! Tests for `[handle] check-len` parsing and its bounds.

use crate::{Config, parse_config};

fn config_from(toml: &str) -> Config {
    parse_config(toml).expect("config should parse")
}

#[test]
fn check_len_is_unset_by_default() {
    assert_eq!(Config::default().handle.check_len, None);
}

#[test]
fn check_len_parses_from_the_kebab_case_key() {
    let config = config_from("[handle]\ncheck-len = 2\n");
    assert_eq!(config.handle.check_len, Some(2));
    config.validate().expect("2 is in range");
}

/// Zero would leave a bare ordinal, which any stale listing satisfies — the
/// exact failure the check exists to catch.
#[test]
fn check_len_zero_is_refused() {
    let err = config_from("[handle]\ncheck-len = 0\n")
        .validate()
        .expect_err("0 should not validate");
    assert!(
        err.contains("handle.check-len"),
        "should name the key: {err}"
    );
}

/// Past six characters a 32-bit hash has nothing left to say, so the extra
/// characters would be constant and read as a guard they are not.
#[test]
fn check_len_beyond_the_hash_width_is_refused() {
    let err = config_from("[handle]\ncheck-len = 7\n")
        .validate()
        .expect_err("7 should not validate");
    assert!(
        err.contains("handle.check-len"),
        "should name the key: {err}"
    );
}

#[test]
fn an_unknown_handle_key_is_refused() {
    assert!(
        parse_config("[handle]\ncheck-length = 2\n").is_err(),
        "a misspelled key must fail rather than being ignored"
    );
}
