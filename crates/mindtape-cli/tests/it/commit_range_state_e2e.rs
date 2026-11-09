//! End-to-end tests for a range reading each commit's own state: its template
//! config, its staged tasks and its config.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::{tempdir, write_file};

use crate::commit_range_e2e::check_range;
use crate::support::{commit, commit_repo, git, git_commit_all};

/// A range builds each commit's record from the config and tasks that commit
/// carries, not from the working tree's.
#[test]
fn a_range_classifies_each_commit_with_its_own_template_config() {
    let name_task = "[commits]\ntrailer = \"Refs\"\n\n\
         [[commits.rules]]\nname = \"name-task-in-refs-trailer\"\nscope = \"commit\"\n\
         when = '!tasks_only'\nassert = 'has(refs)'\n\
         message = \"name the task this commit works on\"\nseverity = \"error\"\n";
    let config =
        format!("[tasks]\nroots = [\".\"]\n\n[templates]\ndirs = [\"templates\"]\n\n{name_task}");
    let marker =
        "#metadata((marker: \"mindtape.task\", fields: (title: \"Parser\"), extras: (:)))\n";
    let dir = tempdir();
    let root = dir.path();
    write_file(root, ".mindtape/config.toml", &config);
    write_file(root, "tasks/parser.typ", marker);
    git_commit_all(root);

    // The commit renames the task into the template directory and names no
    // task; the worktree then points its templates elsewhere.
    std::fs::create_dir_all(root.join("templates")).unwrap();
    git(root, &["mv", "tasks/parser.typ", "templates/parser.typ"]);
    commit(root, "refactor: move the parser");
    write_file(
        root,
        ".mindtape/config.toml",
        config.replace("dirs = [\"templates\"]", "dirs = [\"other-templates\"]"),
    );

    let (ok, text) = check_range(root, "HEAD~1..HEAD");
    assert!(
        !ok,
        "the commit's own template config makes the rename non-task-only: {text}"
    );
    assert!(
        text.contains("commits.rules.name-task-in-refs-trailer"),
        "{text}"
    );
}

#[test]
fn a_range_reads_each_commits_trailer_key() {
    let rules = "[commits]\ntrailer = \"Task\"\n\n\
         [[commits.rules]]\nname = \"name-task-in-refs-trailer\"\nscope = \"commit\"\n\
         when = '!tasks_only'\nassert = 'has(refs)'\n\
         message = \"name the task this commit works on\"\nseverity = \"error\"\n";
    let dir = commit_repo(rules);
    let root = dir.path();

    write_file(root, "src/lib.rs", "work\n");
    git(root, &["add", "src/lib.rs"]);
    commit(root, "feat: parse\n\nTask: parser");

    // The working tree keeps the old key, which the checked commit does not
    // carry: the range must parse the message with the commit's own config.
    let config = std::fs::read_to_string(root.join(".mindtape/config.toml"))
        .unwrap()
        .replace("trailer = \"Task\"", "trailer = \"Refs\"");
    write_file(root, ".mindtape/config.toml", &config);

    let (ok, text) = check_range(root, "HEAD~1..HEAD");
    assert!(
        ok,
        "the commit's own `Task:` trailer names its task: {text}"
    );
}

#[test]
fn a_range_reads_each_commits_staged_tasks_and_config() {
    let dir = commit_repo("");
    let root = dir.path();
    let working = std::fs::read_to_string(root.join(".mindtape/config.toml")).unwrap();

    // The checked commit carries an external-ref pattern and a new task.
    let staged = format!("{working}\n[commits]\nexternal-refs = [\"^EXT-[0-9]+$\"]\n");
    write_file(root, ".mindtape/config.toml", &staged);
    write_file(
        root,
        "tasks/fresh.typ",
        mindtape::typst_task!("(\n  title: \"Fresh\",\n  status: wip(2026, 6, 1),\n)\n"),
    );
    git(root, &["add", "-A"]);
    commit(root, "feat: fresh\n\nRefs: fresh EXT-7");

    // The working tree keeps the config without the pattern.
    write_file(root, ".mindtape/config.toml", &working);

    let (ok, text) = check_range(root, "HEAD~1..HEAD");
    assert!(
        ok,
        "the commit's own config admits its external ref: {text}"
    );
}
