//! `mt explain add`'s text rendering over a config with no recorded origins.
#![allow(clippy::unwrap_used)]

use mindtape::active_add;

use super::render;

fn config(hooks: &str) -> mindtape::Config {
    mindtape::config::parse_config(hooks).unwrap()
}

#[test]
fn a_hook_prints_its_label_when_actions_and_origin() {
    let config = config(
        r#"
[[on.add]]
name = "fixes"
when = 'template == "task"'
actions = [{ dir = "tasks/fixes" }]
"#,
    );
    let hooks = active_add(&config, &[]);
    let text = render(&hooks);
    for needle in [
        "on.add.fixes",
        "template == \"task\"",
        "dir = \"tasks/fixes\"",
        "built-in",
    ] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}

#[test]
fn no_hooks_says_so() {
    assert_eq!(render(&[]), "no add hooks apply\n");
}
