//! `active_add` over a config with no recorded origins.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::active_add;

#[test]
fn a_config_without_origins_keeps_every_hook_by_position() {
    let config = crate::config::parse_config(
        r#"
[[on.add]]
actions = [{ dir = "tasks/a" }]

[[on.add]]
name = "second"
actions = [{ set = { kind = "feat" } }]
"#,
    )
    .unwrap();

    let hooks = active_add(&config, &[]);
    assert_eq!(hooks.len(), 2);
    assert_eq!(hooks[0].label, "on.add[0]");
    assert_eq!(hooks[1].label, "on.add.second");
    assert_eq!(hooks[1].origin.to_string(), "built-in");
}
