//! A rule skipped because `changed()` cannot read git is a breach of the
//! config, reported once at its entry. The template check, which runs the
//! same rules over each template, does not repeat it.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;
use mindtape::test_support::write_file;

use support::{mt, project_with_tasks};

#[test]
fn a_pack_rule_skipped_outside_git_is_reported_once() {
    let dir = project_with_tasks(&[("parser.typ", "Write the parser")]);
    let config = dir.path().join(".mindtape/config.toml");
    let base = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, format!("extends = [\"./pack.toml\"]\n{base}")).unwrap();
    write_file(
        dir.path(),
        "pack.toml",
        "[[tasks.rules]]\nname = \"src-budget\"\nassert = 'changed(\"worktree\") <= 3'\n",
    );
    let out = mt(dir.path()).arg("check").output().unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(text.matches("skipped").count(), 1, "{text}");
    assert!(text.contains("pack.toml:1"), "{text}");
}
