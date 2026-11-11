//! `[[on.add]]` merges by name across packs and layers like the other rule
//! arrays: a pack's hook joins the project's, a later same-name entry replaces
//! it in place, and `disable` drops it by name.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use crate::config::Config;
use crate::config::add_hooks::AddValue;
use crate::config::layers::load_config_layered_from;
use crate::test_support::{home_and_project, write_file};

/// One `[[on.add]]` entry named `name` (unnamed when empty), setting `kind`.
fn hook(name: &str, kind: &str) -> String {
    let name = match name {
        "" => String::new(),
        name => format!("name = \"{name}\"\n"),
    };
    format!("[[on.add]]\n{name}actions = [{{ set = {{ kind = \"{kind}\" }} }}]\n")
}

/// `(name, kind)` for each merged add hook.
fn summary(config: &Config) -> Vec<(Option<&str>, Option<&str>)> {
    let mut rows = Vec::new();
    for entry in &config.on.add {
        let kind = match entry
            .actions
            .first()
            .and_then(|action| action.set.get("kind"))
        {
            Some(AddValue::Text(text)) => Some(text.as_str()),
            _ => None,
        };
        rows.push((entry.name.as_deref(), kind));
    }
    rows
}

/// A project extending `team.toml` (holding `pack`) with `project` config.
fn config_with_pack(pack: &str, project: &str) -> Config {
    let (_tmp, home, root) = home_and_project();
    write_file(&home, "team.toml", pack);
    write_file(
        &root,
        ".mindtape/config.toml",
        format!("extends = [\"../team.toml\"]\n{project}"),
    );
    load_config_layered_from(&root, Some(&home), None)
        .expect("config loads")
        .0
}

#[test]
fn a_pack_hook_joins_and_a_named_project_hook_replaces() {
    let joined = config_with_pack(
        &hook("packed", "from-pack"),
        &hook("theirs", "from-project"),
    );
    assert_eq!(
        summary(&joined),
        [
            (Some("packed"), Some("from-pack")),
            (Some("theirs"), Some("from-project")),
        ]
    );

    let pack = [hook("placement", "from-pack"), hook("other", "other")].concat();
    let replaced = config_with_pack(&pack, &hook("placement", "from-project"));
    assert_eq!(
        summary(&replaced),
        [
            (Some("placement"), Some("from-project")),
            (Some("other"), Some("other")),
        ]
    );
}

#[test]
fn disable_drops_a_pack_hook_by_name() {
    let pack = [hook("placement", "from-pack"), hook("other", "other")].concat();
    let config = config_with_pack(&pack, "disable = [\"placement\"]\n");

    assert_eq!(summary(&config), [(Some("other"), Some("other"))]);
}

#[test]
fn two_entries_of_one_file_sharing_a_name_are_a_config_error() {
    let twice = [hook("placement", "a"), hook("placement", "b")].concat();
    let (_tmp, home, root) = home_and_project();
    write_file(&root, ".mindtape/config.toml", twice);

    let err = load_config_layered_from(&root, Some(&home), None)
        .expect_err("a duplicate name fails the load")
        .to_string();

    assert!(
        err.contains("on.add[1]: duplicate name 'placement'"),
        "{err}"
    );
}
