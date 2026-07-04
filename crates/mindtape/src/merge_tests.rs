#![allow(clippy::unwrap_used)]

use super::deep_merge;

#[test]
fn tables_merge_field_by_field() {
    let mut base: toml::Value = toml::from_str("a = 1\nb = 2\n").unwrap();
    let overlay: toml::Value = toml::from_str("b = 3\nc = 4\n").unwrap();
    deep_merge(&mut base, overlay);
    assert_eq!(
        base,
        toml::from_str::<toml::Value>("a = 1\nb = 3\nc = 4\n").unwrap()
    );
}

#[test]
fn nested_tables_recurse() {
    let mut base: toml::Value = toml::from_str("[x]\na = 1\nb = 2\n").unwrap();
    let overlay: toml::Value = toml::from_str("[x]\nb = 9\n").unwrap();
    deep_merge(&mut base, overlay);
    assert_eq!(
        base,
        toml::from_str::<toml::Value>("[x]\na = 1\nb = 9\n").unwrap()
    );
}

#[test]
fn arrays_replace_wholesale() {
    let mut base: toml::Value = toml::from_str("a = [1, 2, 3]\n").unwrap();
    let overlay: toml::Value = toml::from_str("a = [9]\n").unwrap();
    deep_merge(&mut base, overlay);
    assert_eq!(base, toml::from_str::<toml::Value>("a = [9]\n").unwrap());
}

#[test]
fn scalars_replace() {
    let mut base: toml::Value = toml::from_str("a = 1\n").unwrap();
    let overlay: toml::Value = toml::from_str("a = 2\n").unwrap();
    deep_merge(&mut base, overlay);
    assert_eq!(base, toml::from_str::<toml::Value>("a = 2\n").unwrap());
}

#[test]
fn overlay_table_replaces_non_table_base() {
    let mut base: toml::Value = toml::Value::Integer(1);
    let overlay: toml::Value = toml::from_str("a = 1\n").unwrap();
    deep_merge(&mut base, overlay);
    assert_eq!(base, toml::from_str::<toml::Value>("a = 1\n").unwrap());
}
