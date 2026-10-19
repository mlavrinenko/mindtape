#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::{Path, PathBuf};

use mindtape::test_support::{TASK_SRC, tempdir, write_bare_project, write_file};
use mindtape::{LoadError, LoadOutcome};

use super::{member_load_problems, member_warnings, open_set, open_set_warned};
use crate::diag::{Kind, plain};
use crate::util::{Project, held_back_warning};

/// A bare project at `dir/name` holding one task named `stem`.
fn member(dir: &Path, name: &str, stem: &str) -> (String, PathBuf) {
    let root = dir.join(name);
    write_bare_project(&root);
    write_file(&root, format!("tasks/{stem}.typ"), TASK_SRC);
    (name.to_owned(), root)
}

#[test]
fn a_set_opens_in_the_order_given_and_a_broken_config_fails_only_its_member() {
    let dir = tempdir();
    let broken = member(dir.path(), "broken", "lost");
    write_file(&broken.1, ".mindtape/config.toml", "[tasks\n");
    let members = vec![
        member(dir.path(), "zeta", "z"),
        broken,
        member(dir.path(), "alpha", "a"),
    ];

    let set = open_set(&members);

    let names: Vec<&str> = set.iter().map(|member| member.name.as_str()).collect();
    assert_eq!(names, ["zeta", "broken", "alpha"]);
    assert_eq!(set[0].project.as_ref().unwrap().outcome.tasks[0].id, "z");
    assert_eq!(set[0].project.as_ref().unwrap().root, members[0].1);
    assert_eq!(set[1].project.as_ref().unwrap_err().kind(), Kind::Failed);
    assert_eq!(set[2].project.as_ref().unwrap().outcome.tasks[0].id, "a");
}

#[test]
fn a_members_config_warning_and_held_commands_name_it() {
    let root = PathBuf::from("/member-warnings");
    let config = mindtape::parse_config(
        "[tasks.fields.x-machine]\nvalues = [\"alpha\"]\n\
         [[tasks.rules]]\nname = \"r\"\nassert = 'x-machnie == \"alpha\"'\n",
    )
    .unwrap();
    mindtape::allow::hold(&root, &["make up".to_owned()]);
    let project = Project {
        root,
        config,
        outcome: LoadOutcome::default(),
        hyperlinks: false,
    };

    let texts: Vec<String> = member_warnings("api", &project).iter().map(plain).collect();

    assert_eq!(texts.len(), 2, "{texts:?}");
    assert!(texts[0].starts_with("warning: .mindtape/config.toml: tasks.rules[0]"));
    assert!(texts[0].ends_with("note: project: api\n"), "{}", texts[0]);
    assert!(
        texts[1].contains("hint: read them, then run mt @api allow\n"),
        "{}",
        texts[1]
    );
    assert!(texts[1].ends_with("note: project: api\n"), "{}", texts[1]);
    assert!(!mindtape::allow::held_back().contains(&"make up".to_owned()));
}

#[test]
fn a_member_with_nothing_to_say_warns_nothing() {
    let project = Project {
        root: PathBuf::from("/member-quiet"),
        config: mindtape::Config::default(),
        outcome: LoadOutcome::default(),
        hyperlinks: false,
    };

    assert!(member_warnings("quiet", &project).is_empty());
}

#[test]
fn a_members_load_error_names_it() {
    let outcome = LoadOutcome {
        errors: vec![LoadError {
            file: "tasks/broken.typ".into(),
            line: Some(2),
            message: "eval failed".to_owned(),
            ..LoadError::default()
        }],
        ..LoadOutcome::default()
    };

    let diags = member_load_problems("web", &outcome, Kind::Warning);

    assert_eq!(
        diags.iter().map(plain).collect::<Vec<_>>(),
        ["warning: tasks/broken.typ:2: eval failed\nnote: project: web\n"]
    );
}

#[test]
fn a_members_held_back_warning_asks_for_mt_at_its_name_allow() {
    let msg = plain(
        &held_back_warning(&["b".to_owned()])
            .unwrap()
            .in_member("web"),
    );

    assert_eq!(
        msg,
        "warning: not running the commands your config declares until you allow them\n\
         hint: read them, then run mt @web allow\nnote: command: 'b'\nnote: project: web\n"
    );
}

#[test]
fn a_warned_set_keeps_each_members_load_errors_and_warns_its_config() {
    let dir = tempdir();
    let noisy = member(dir.path(), "noisy", "fine");
    write_file(
        &noisy.1,
        ".mindtape/config.toml",
        "[tasks]\nroots = [\"tasks\"]\n[tasks.fields.x-machine]\nvalues = [\"alpha\"]\n\
         [[tasks.rules]]\nname = \"r\"\nassert = 'x-machnie == \"alpha\"'\n",
    );
    write_file(
        &noisy.1,
        "tasks/broken.typ",
        "#metadata((marker: \"mindtape.task\", fields: (",
    );

    let set = open_set_warned(&[noisy]);

    let project = set[0].project.as_ref().unwrap();
    assert_eq!(project.outcome.tasks[0].id, "fine");
    assert_eq!(project.outcome.errors.len(), 1);
}
