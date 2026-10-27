#![allow(clippy::unwrap_used)]

use crate::test_support::tempdir;
use std::path::{Path, PathBuf};

use super::{
    AllowError, allowed_in, commands, digest, held_back, hold, record_in, record_path, repo_key,
    runnable_editor_in, take_held,
};
use crate::config::Config;
use crate::waits::Checker;

fn config(runs: &[&str]) -> Config {
    let mut config = Config::default();
    config.tasks.waits = runs
        .iter()
        .enumerate()
        .map(|(index, run)| Checker {
            name: format!("c{index}"),
            run: (*run).to_owned(),
        })
        .collect();
    config
}

fn lines(runs: &[&str]) -> Vec<String> {
    runs.iter().map(|run| (*run).to_owned()).collect()
}

#[test]
fn the_commands_are_every_run_line_sorted_once() {
    assert_eq!(commands(&config(&["b", "a", "b"])), lines(&["a", "b"]));
    assert!(commands(&Config::default()).is_empty());
}

#[test]
fn the_commands_take_each_flip_run_line_beside_the_checkers() {
    let config = crate::parse_config(
        "[[tasks.waits]]\nname = \"up\"\nrun = \"b\"\n\
         [[on.flip]]\nwhen = ''\nactions = [{ run = \"c\", gate = true }, { show = \"x\" }]\n\
         [[on.flip]]\nwhen = ''\nactions = [{ run = \"b\" }, { gate = true }, { run = \"a\" }]\n",
    )
    .unwrap();

    assert_eq!(commands(&config), lines(&["a", "b", "c"]));
}

#[test]
fn taking_a_projects_held_commands_leaves_the_others_held() {
    hold(
        Path::new("taken"),
        &lines(&["held-taken-b", "held-taken-a"]),
    );
    hold(Path::new("kept"), &lines(&["held-kept"]));

    assert_eq!(
        take_held(Path::new("taken")),
        ["held-taken-a", "held-taken-b"]
    );

    let held = held_back();
    assert!(held.contains(&"held-kept".to_owned()));
    assert!(!held.iter().any(|line| line.starts_with("held-taken-")));
    assert!(take_held(Path::new("taken")).is_empty());
}

#[test]
fn a_held_command_is_listed_once() {
    hold(Path::new("one"), &lines(&["held-once-z", "held-once-y"]));
    hold(Path::new("two"), &lines(&["held-once-z"]));

    let held = held_back();

    assert_eq!(
        held.iter()
            .filter(|line| line.starts_with("held-once-"))
            .collect::<Vec<_>>(),
        ["held-once-y", "held-once-z"]
    );
}

#[test]
fn the_digest_changes_with_any_line_and_cannot_be_forged_by_joining_two() {
    let one = digest(&lines(&["a b"]));
    assert_eq!(one.len(), 40);
    assert_ne!(one, digest(&lines(&["a c"])));
    assert_ne!(one, digest(&lines(&["a", "b"])));
    assert_ne!(digest(&lines(&["ab"])), digest(&lines(&["a", "b"])));
}

#[test]
fn outside_git_the_key_is_the_root() {
    let dir = tempdir();

    assert_eq!(repo_key(dir.path()), crate::canonical(dir.path()).unwrap());
}

#[test]
fn inside_git_every_worktree_shares_the_common_dir_as_key() {
    let dir = tempdir();
    let main = dir.path().join("main");
    std::fs::create_dir_all(main.join("sub")).unwrap();
    crate::test_support::git_init(&main);
    crate::test_support::git_as_tester(&main, &["commit", "-q", "--allow-empty", "-m", "seed"]);
    let sibling = dir.path().join("sibling");
    crate::test_support::git_as_tester(
        &main,
        &["worktree", "add", "-q", sibling.to_str().unwrap()],
    );

    let key = repo_key(&main);

    assert_eq!(key, crate::canonical(&main.join(".git")).unwrap());
    assert_eq!(repo_key(&main.join("sub")), key);
    assert_eq!(repo_key(&sibling), key);
}

/// A fresh data home and an existing project root, side by side in one
/// tempdir that the caller keeps alive.
fn data_home_and_root() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempdir();
    let home = dir.path().join("home");
    let root = dir.path().join("root");
    std::fs::create_dir_all(&root).unwrap();
    (dir, home, root)
}

#[test]
fn a_recorded_set_is_allowed_and_a_changed_one_is_not() {
    let (_dir, home, root) = data_home_and_root();
    let recorded = lines(&["scripts/locked {id}"]);

    assert!(!allowed_in(Some(&home), &root, &recorded));
    let path = record_in(Some(&home), &root, &recorded).unwrap();

    assert_eq!(path, record_path(&home, &repo_key(&root)));
    assert!(path.starts_with(home.join("mindtape").join("allowed")));
    assert!(allowed_in(Some(&home), &root, &recorded));
    assert!(!allowed_in(
        Some(&home),
        &root,
        &lines(&["scripts/other {id}"])
    ));
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.ends_with("# scripts/locked {id}\n"), "{text}");
}

#[test]
fn no_commands_need_no_approval_and_no_data_home_allows_nothing() {
    let root = Path::new("/no/such/root");

    assert!(allowed_in(None, root, &[]));
    assert!(!allowed_in(None, root, &lines(&["a"])));
    assert!(matches!(
        record_in(None, root, &lines(&["a"])),
        Err(AllowError::NoDataHome)
    ));
}

#[test]
fn record_names_the_file_it_could_not_write() {
    let dir = tempdir();
    let home = dir.path().join("file");
    std::fs::write(&home, "").unwrap();

    let err = record_in(Some(&home), dir.path(), &lines(&["a"])).unwrap_err();

    assert!(matches!(err, AllowError::Write { .. }), "{err}");
    assert!(err.to_string().starts_with("can't write "), "{err}");
}

/// The config of a project whose own `config.toml` is `project`, loaded over
/// a user layer (`<config home>/mindtape/defaults.toml`) holding `user`.
fn layered(user: &str, project: &str) -> Config {
    layered_under_ancestor(user, "", project)
}

/// [`layered`], with an ancestor `.mindtape/defaults.toml` holding `ancestor`
/// one directory above the project, which the walk-up reaches.
fn layered_under_ancestor(user: &str, ancestor: &str, project: &str) -> Config {
    layered_with_pack(user, ancestor, project, "")
}

/// [`layered_under_ancestor`], with `pack` as the project's `team.toml`, a
/// path pack its config can extend as `./team.toml`.
fn layered_with_pack(user: &str, ancestor: &str, project: &str, pack: &str) -> Config {
    let dir = tempdir();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(dir.path().join(".mindtape").join("defaults.toml"), ancestor).unwrap();
    let xdg = dir.path().join("xdg");
    std::fs::create_dir_all(xdg.join("mindtape")).unwrap();
    std::fs::write(xdg.join("mindtape").join("defaults.toml"), user).unwrap();
    let root = dir.path().join("root");
    std::fs::create_dir_all(root.join(".mindtape")).unwrap();
    std::fs::write(root.join(".mindtape").join("config.toml"), project).unwrap();
    std::fs::write(root.join("team.toml"), pack).unwrap();
    crate::load_config_layered_from(&root, Some(dir.path()), Some(&xdg))
        .unwrap()
        .0
}

#[test]
fn the_commands_list_a_go_and_an_add_editor_beside_the_run_lines() {
    let config = crate::parse_config(
        "[go]\neditor = \"zed {task}\"\n[add]\neditor = \"nvim -p\"\n\
         [[tasks.waits]]\nname = \"up\"\nrun = \"b\"\n",
    )
    .unwrap();

    assert_eq!(commands(&config), lines(&["b", "nvim -p", "zed {task}"]));
}

/// `commands` of a project over a user layer, for each `(user, project, listed)`
/// case: the editors that still ask for `mt allow`.
#[test]
fn only_an_editor_the_users_own_layer_did_not_set_is_listed() {
    let cases = [
        ("", "[go]\neditor = \"  \"\n", &[][..]),
        (
            "",
            "[go]\neditor = \"evil {task}\"\n[add]\neditor = \"evil\"\n",
            &["evil", "evil {task}"],
        ),
        (
            "[go]\neditor = \"mine {task}\"\n[add]\neditor = \"mine\"\n",
            "",
            &[],
        ),
        (
            "[go]\neditor = \"mine {task}\"\n",
            "[go]\neditor = \"evil {task}\"\n",
            &["evil {task}"],
        ),
    ];
    for (user, project, listed) in cases {
        assert_eq!(
            commands(&layered(user, project)),
            lines(listed),
            "{user:?} {project:?}"
        );
    }
}

#[test]
fn an_ancestors_defaults_editor_is_listed_like_the_projects() {
    let config = layered_under_ancestor(
        "[go]\neditor = \"mine {task}\"\n",
        "[go]\neditor = \"ancestor {task}\"\n[add]\neditor = \"ancestor\"\n",
        "",
    );

    assert_eq!(commands(&config), lines(&["ancestor", "ancestor {task}"]));
}

#[test]
fn a_packs_editor_is_listed_and_held_like_the_projects() {
    let config = layered_with_pack(
        "[go]\neditor = \"mine {task}\"\n",
        "",
        "extends = [\"./team.toml\"]\n",
        "[go]\neditor = \"pack {task}\"\n[add]\neditor = \"pack\"\n",
    );
    assert_eq!(commands(&config), lines(&["pack", "pack {task}"]));

    let root = Path::new("/no/such/pack-root");
    let editor = config.go.editor.as_deref();
    assert_eq!(runnable_editor_in(None, root, &config, editor), None);
    assert_eq!(take_held(root), ["pack {task}"]);
}

#[test]
fn an_editor_is_held_until_the_commands_are_allowed() {
    let (_dir, home, root) = data_home_and_root();
    let config = layered("", "[go]\neditor = \"held-editor-evil {task}\"\n");
    let editor = config.go.editor.as_deref();

    assert_eq!(
        runnable_editor_in(Some(&home), &root, &config, editor),
        None
    );
    assert_eq!(take_held(&root), ["held-editor-evil {task}"]);

    record_in(Some(&home), &root, &commands(&config)).unwrap();

    assert_eq!(
        runnable_editor_in(Some(&home), &root, &config, editor).as_deref(),
        Some("held-editor-evil {task}")
    );
    assert!(take_held(&root).is_empty());
}

#[test]
fn a_user_layer_editor_runs_unasked_and_no_editor_holds_nothing() {
    let root = Path::new("/no/such/root");
    let config = layered("[go]\neditor = \"mine {task}\"\n", "");

    assert_eq!(
        runnable_editor_in(None, root, &config, config.go.editor.as_deref()).as_deref(),
        Some("mine {task}")
    );
    assert_eq!(runnable_editor_in(None, root, &config, None), None);
    assert_eq!(runnable_editor_in(None, root, &config, Some("  ")), None);
    assert!(take_held(root).is_empty());
}
