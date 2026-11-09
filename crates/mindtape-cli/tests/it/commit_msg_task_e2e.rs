//! End-to-end tests for `scope = "task"` commit rules
//! (`truth/tasks/evaluate-task-scope-commit-rules.typ`): a rule run per task
//! the commit changes or names, reading that task as of `HEAD`, the index and
//! the working tree through `head(...)`, `staged(...)` and `worktree(...)`.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::{tempdir, write_file};
use std::path::Path;

use crate::support;

use support::{check_commit_msg, commit_repo, git, git_commit_all, run_text, scaffold_project};

/// The task's worked example: a start or a close rides with the work it
/// claims, unless the commit is a decision's own.
const FLIP_WITH_WORK: &str = "[[commits.rules]]\n\
     name = \"flip-task-with-its-work\"\n\
     scope = \"task\"\n\
     when = 'into(\"wip\") || into(\"done\")'\n\
     assert = 'named && (!tasks_only || staged(has_tag(\"decision\")))'\n\
     message = \"start or close a task in a commit that carries its work\"\n\
     severity = \"error\"\n";

/// The shipped rule: a commit that carries work outside task files names the
/// task it works on in the configured trailer.
const NAME_TASK: &str = "[commits]\ntrailer = \"Refs\"\n\n\
     [[commits.rules]]\n\
     name = \"name-task-in-refs-trailer\"\n\
     scope = \"commit\"\n\
     when = '!tasks_only'\n\
     assert = 'has(refs)'\n\
     message = \"name the task this commit works on in a `Refs: <stem>` footer\"\n\
     severity = \"error\"\n";

/// A committed repository with `config` and each `(path, source)` file.
fn raw_repo(config: &str, files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempdir();
    write_file(dir.path(), ".mindtape/config.toml", config);
    for (path, source) in files {
        write_file(dir.path(), path, source);
    }
    git_commit_all(dir.path());
    dir
}

/// A root-`.` config naming `templates` as its template directory.
fn root_dot_config(templates: &str) -> String {
    format!("[tasks]\nroots = [\".\"]\n\n[templates]\ndirs = [\"{templates}\"]\n\n{NAME_TASK}")
}

/// Rewrite `tasks/parser.typ` with `status` and any `extra` fields.
fn write_parser(dir: &Path, status: &str, extra: &str) {
    write_file(
        dir,
        "tasks/parser.typ",
        format!(
            "#import \"@local/mindtape:0.4.0\": *\n\n\
             #show: task.with(\n  title: \"Write the parser\",\n  status: {status},\n  \
             priority: 3,\n{extra})\n"
        ),
    );
}

/// Close `parser` and stage the close.
fn stage_close(dir: &Path, extra: &str) {
    write_parser(dir, "done(2026, 6, 2)", extra);
    git(dir, &["add", "tasks/parser.typ"]);
}

/// Stage one new work file.
fn stage_work(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    write_file(dir, "src/lib.rs", "work\n");
    git(dir, &["add", "src/lib.rs"]);
}

#[test]
fn an_unsupported_head_still_classifies_the_index_state() {
    let config = format!(
        "[tasks]\nroots = [\"tasks\"]\n\n{NAME_TASK}\n\n\
         [[commits.rules]]\nname = \"task-rule\"\nscope = \"task\"\n\
         assert = 'named'\nseverity = \"error\"\n"
    );
    let old = "#import \"@local/mindtape:0.1.0\": *\n\n\
               #show: task.with(title: \"Old\", status: wip(2026, 6, 1))\n";
    let dir = raw_repo(&config, &[("tasks/old.typ", old)]);
    write_file(
        dir.path(),
        "tasks/fresh.typ",
        "#metadata((marker: \"mindtape.task\", fields: (title: \"Fresh\"), extras: (:)))\n",
    );
    git(dir.path(), &["add", "tasks/fresh.typ"]);
    stage_work(dir.path());

    let (ok, text) = check_commit_msg(dir.path(), "feat: fresh\n\nRefs: fresh\n");
    assert!(
        ok,
        "the index state, not the unsupported HEAD state, classifies the commit: {text}"
    );
}

#[test]
fn the_index_trailer_key_parses_the_message() {
    let dir = commit_repo(NAME_TASK);
    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml"))
        .unwrap()
        .replace("trailer = \"Refs\"", "trailer = \"Task\"");
    write_file(dir.path(), ".mindtape/config.toml", &config);
    git(dir.path(), &["add", ".mindtape/config.toml"]);
    stage_work(dir.path());

    let (ok, text) = check_commit_msg(dir.path(), "feat: parse\n\nTask: parser\n");
    assert!(
        ok,
        "the index names `Task`, so the message's `Task:` trailer is read: {text}"
    );
}

#[test]
fn a_close_only_commit_is_refused_naming_the_task() {
    let dir = commit_repo(FLIP_WITH_WORK);
    stage_close(dir.path(), "");
    let text = support::commit_msg_refusal(dir.path(), "task: close the parser\n");
    assert!(
        text.contains("commits.rules.flip-task-with-its-work: parser: start or close"),
        "the breach names the rule, the task and the message: {text}"
    );
}

#[test]
fn a_close_riding_with_its_work_passes() {
    let dir = commit_repo(FLIP_WITH_WORK);
    stage_close(dir.path(), "");
    stage_work(dir.path());
    let (ok, text) = check_commit_msg(dir.path(), "feat: parse\n\nRefs: parser\n");
    assert!(ok, "a close beside the work it claims passes: {text}");
}

#[test]
fn a_work_commit_leaving_the_status_alone_is_outside_the_rule() {
    let dir = commit_repo(FLIP_WITH_WORK);
    write_parser(dir.path(), "wip(2026, 6, 1)", "  tags: (\"x\",),\n");
    git(dir.path(), &["add", "tasks/parser.typ"]);
    let (ok, text) = check_commit_msg(dir.path(), "task: tag the parser\n");
    assert!(ok, "no flip, so `into` never holds: {text}");
}

#[test]
fn an_estimate_added_in_the_closing_commit_is_read_through_staged() {
    let rules = "[[commits.rules]]\n\
         name = \"closed-with-estimate\"\n\
         scope = \"task\"\n\
         when = 'into(\"done\")'\n\
         assert = 'staged(has(estimate)) && !head(has(estimate))'\n\
         severity = \"error\"\n";
    let dir = commit_repo(rules);
    stage_close(dir.path(), "  estimate: (days: 1),\n");
    let (ok, text) = check_commit_msg(dir.path(), "task: close the parser\n");
    assert!(ok, "the staged close carries the estimate: {text}");

    // The same estimate only in the working tree: the index lacks it.
    stage_close(dir.path(), "");
    write_parser(dir.path(), "done(2026, 6, 2)", "  estimate: (days: 1),\n");
    let text = support::commit_msg_refusal(dir.path(), "task: close the parser\n");
    assert!(
        text.contains("commits.rules.closed-with-estimate"),
        "{text}"
    );
}

#[test]
fn a_flip_left_unstaged_warns() {
    let rules = "[[commits.rules]]\n\
         name = \"flip-unstaged\"\n\
         scope = \"task\"\n\
         when = 'worktree(status) != staged(status)'\n\
         assert = 'miss(refs)'\n\
         message = \"stage the flip with its work\"\n";
    let dir = commit_repo(rules);
    stage_work(dir.path());
    write_parser(dir.path(), "done(2026, 6, 2)", "");
    let (ok, text) = check_commit_msg(dir.path(), "feat: parse\n\nRefs: parser\n");
    assert!(ok, "a warn rule reports without failing: {text}");
    assert!(
        text.contains("warning:") && text.contains("flip-unstaged: parser: stage the flip"),
        "the warning names the task: {text}"
    );
}

#[test]
fn a_deleted_task_does_not_exist_in_the_staged_state() {
    let rules = "[[commits.rules]]\n\
         name = \"no-deleting-open-tasks\"\n\
         scope = \"task\"\n\
         when = 'head(exists) && !staged(exists)'\n\
         assert = 'head(terminal)'\n\
         severity = \"error\"\n";
    let dir = commit_repo(rules);
    git(dir.path(), &["rm", "-q", "tasks/parser.typ"]);
    let text = support::commit_msg_refusal(dir.path(), "task: drop the parser\n");
    assert!(text.contains("no-deleting-open-tasks: parser"), "{text}");
}

#[test]
fn a_bare_task_field_in_a_task_rule_is_refused_at_load() {
    let rules = "[[commits.rules]]\n\
         name = \"bare\"\n\
         scope = \"task\"\n\
         assert = 'status == \"done\"'\n";
    let dir = commit_repo(rules);
    let text = support::commit_msg_refusal(dir.path(), "feat: x\n");
    assert!(
        text.contains("commits.rules[0].assert") && text.contains("staged("),
        "the refusal names the state wrappers: {text}"
    );
}

#[test]
fn a_state_wrapper_in_a_commit_rule_is_refused_at_load() {
    let rules = "[[commits.rules]]\n\
         name = \"wrapped\"\n\
         assert = 'staged(status == \"done\")'\n";
    let dir = commit_repo(rules);
    let text = support::commit_msg_refusal(dir.path(), "feat: x\n");
    assert!(text.contains("scope = \"task\""), "{text}");
}

#[test]
fn into_naming_no_status_is_refused_at_load() {
    let rules = "[[commits.rules]]\n\
         scope = \"task\"\n\
         when = 'into(\"dnoe\")'\n\
         assert = 'named'\n";
    let dir = commit_repo(rules);
    let text = support::commit_msg_refusal(dir.path(), "feat: x\n");
    assert!(text.contains("dnoe"), "{text}");
}

/// A prelude stub whose marker always carries `title`, so a loaded task's
/// title says which package tree served it.
fn titled_prelude(title: &str) -> String {
    format!(
        "#let task(body, ..meta) = [#metadata((marker: \"mindtape.task\", \
         fields: (title: \"{title}\"), extras: (:))) <mindtape>]\n"
    )
}

#[test]
fn a_first_commit_with_a_pin_passes() {
    let rules = "[[commits.rules]]\n\
         name = \"staged-loads\"\nscope = \"task\"\nwhen = 'named'\n\
         assert = 'staged(exists)'\nseverity = \"error\"\n";
    let dir = scaffold_project(&format!("[tasks]\nroots = [\"tasks\"]\n{rules}"));
    write_file(
        dir.path(),
        "tasks/parser.typ",
        "#import \"@local/mindtape:0.4.0\": *\n\n         #show: task.with(title: \"Write the parser\", status: wip(2026, 6, 1))\n",
    );
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "-A"]);
    let message = dir.path().join("COMMIT_MSG");
    std::fs::write(&message, "feat: start\n\nRefs: parser\n").unwrap();

    let (ok, text) = run_text(
        dir.path(),
        &["check", "--commit-msg", message.to_str().unwrap()],
    );
    assert!(ok, "a first commit reads an empty HEAD: {text}");
}

#[test]
fn a_subproject_index_state_reads_its_own_config_and_package() {
    let dir = tempdir();
    let root = dir.path();
    let sub = root.join("sub");
    let manifest =
        "[package]\nname = \"mindtape\"\nversion = \"0.3.0\"\nentrypoint = \"prelude.typ\"\n";
    let old = titled_prelude("Old pin");
    let old_config = "[tasks]\nroots = [\"tasks\"]\n\n[[typst.local-packages]]\nname = \"mindtape\"\npath = \"old-lib\"\n";
    write_file(&sub, "old-lib/typst.toml", manifest);
    write_file(&sub, "old-lib/prelude.typ", &old);
    write_file(&sub, ".mindtape/config.toml", old_config);
    write_file(
        &sub,
        "tasks/parser.typ",
        "#import \"@local/mindtape:0.3.0\": *\n\n#show: task.with(title: \"Parser\")\n",
    );
    git_commit_all(root);

    // The index moves the pin to `lib` and carries the rule; the working tree
    // keeps different bytes behind that pin's path.
    let new = titled_prelude("New pin");
    let rule = "[[commits.rules]]\n\
         name = \"pinned\"\nscope = \"task\"\nwhen = 'named'\n\
         assert = 'head(title == \"Old pin\") && staged(title == \"New pin\")'\n\
         severity = \"error\"\n";
    write_file(&sub, "lib/typst.toml", manifest);
    write_file(&sub, "lib/prelude.typ", &new);
    write_file(
        &sub,
        ".mindtape/config.toml",
        format!(
            "[tasks]\nroots = [\"tasks\"]\n\n\
             [[typst.local-packages]]\nname = \"mindtape\"\npath = \"lib\"\n{rule}"
        ),
    );
    git(root, &["add", "-A"]);
    write_file(&sub, "lib/prelude.typ", titled_prelude("Working pin"));

    let message = root.join("COMMIT_MSG");
    std::fs::write(&message, "feat: move\n\nRefs: parser\n").unwrap();
    let (ok, text) = run_text(
        root,
        &[
            "check",
            "-w",
            "sub",
            "--commit-msg",
            message.to_str().unwrap(),
        ],
    );
    assert!(
        ok,
        "the subproject index state reads its own prefix: {text}"
    );
}

/// A raw task marker with `title`.
fn title_marker(title: &str) -> String {
    format!("#metadata((marker: \"mindtape.task\", fields: (title: \"{title}\"), extras: (:)))\n")
}

#[cfg(unix)]
#[test]
fn a_symlink_task_file_is_not_a_task() {
    let config = "[tasks]\nroots = [\"tasks\"]\n";
    let dir = mindtape::test_support::symlink_repo(
        &[
            (".mindtape/config.toml", config),
            ("shared/real.typ", &title_marker("Linked")),
        ],
        &[("tasks/a.typ", "../shared/real.typ")],
    );
    // T1 stopped treating a symlinked `.typ` as a task source, so the index
    // state holds no `a` for the trailer to name.
    let text = support::commit_msg_refusal(dir.path(), "feat: link\n\nRefs: a\n");
    assert!(text.contains("`a` names no task"), "{text}");
}

#[test]
fn a_template_only_commit_classifies_with_the_index_config() {
    let index = root_dot_config("templates");
    let dir = raw_repo(&index, &[("tasks/parser.typ", &title_marker("Parser"))]);
    let root = dir.path();
    write_file(root, "templates/task.typ", title_marker("Template"));
    git(root, &["add", "templates/task.typ"]);
    // The worktree names no `templates/` directory; the index still does, so
    // the indexed template change is not a task-only commit.
    write_file(
        root,
        ".mindtape/config.toml",
        root_dot_config("other-templates"),
    );
    let text = support::commit_msg_refusal(root, "feat: template\n");
    assert!(
        text.contains("name the task this commit works on"),
        "{text}"
    );
}

#[test]
fn a_rename_into_a_template_directory_needs_a_ref() {
    let config = root_dot_config("templates");
    let dir = raw_repo(&config, &[("tasks/parser.typ", &title_marker("Parser"))]);
    let root = dir.path();
    std::fs::create_dir_all(root.join("templates")).unwrap();
    git(root, &["mv", "tasks/parser.typ", "templates/parser.typ"]);
    let text = support::commit_msg_refusal(root, "refactor: move the parser\n");
    assert!(
        text.contains("name the task this commit works on"),
        "a rename into a template dir is not task-only: {text}"
    );
}

#[test]
fn a_tracked_deleted_task_is_still_a_task_only_commit() {
    let config = root_dot_config("templates");
    let dir = raw_repo(&config, &[("tasks/parser.typ", &title_marker("Parser"))]);
    let root = dir.path();
    git(root, &["rm", "-q", "tasks/parser.typ"]);
    let (ok, text) = check_commit_msg(root, "task: drop the parser\n");
    assert!(ok, "a deleted task path stays a commit-state task: {text}");
}

#[test]
fn a_staged_only_task_file_is_a_commit_state_task() {
    let config = "[tasks]\nroots = [\"tasks\"]\n\n\
         [[commits.rules]]\nname = \"staged-exists\"\nscope = \"task\"\nwhen = 'named'\n\
         assert = 'staged(exists)'\nseverity = \"error\"\n";
    let dir = raw_repo(config, &[]);
    let root = dir.path();
    write_file(root, "tasks/fresh.typ", title_marker("Fresh"));
    git(root, &["add", "tasks/fresh.typ"]);
    std::fs::remove_file(root.join("tasks/fresh.typ")).unwrap();
    let (ok, text) = check_commit_msg(root, "feat: fresh\n\nRefs: fresh\n");
    assert!(ok, "the index holds the only copy of the task: {text}");
}

#[test]
fn a_path_form_naming_no_task_contributes_no_ref() {
    let dir = commit_repo(NAME_TASK);
    stage_work(dir.path());
    let text = support::commit_msg_refusal(dir.path(), "feat: parse\n\nRefs: tasks/gone.typ\n");
    assert!(text.contains("names no task"), "{text}");
    assert!(
        text.contains("commits.rules.name-task-in-refs-trailer"),
        "the path must contribute no ref, so the rule sees none: {text}"
    );
}
