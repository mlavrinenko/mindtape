//! Shared scaffolding for CLI end-to-end tests (`tests/*_e2e.rs`).
//!
//! Not a test target itself: cargo only auto-discovers `.rs` files directly
//! under `tests/`, so this module only compiles into a binary that `mod
//! support;`s it — and each such binary sees the helpers it does not call as
//! dead code, hence the module-wide allow.
#![allow(clippy::unwrap_used, clippy::disallowed_methods, dead_code)]

use std::path::{Path, PathBuf};

const PRELUDE: &str = include_str!("../../../../lib/prelude.typ");

/// Where [`isolated_env`] points the global-config lookup: a path under the
/// test's own project directory that nothing ever creates.
const NO_GLOBAL_CONFIG: &str = "no-global-config";

/// Environment variables that steer `mt` and must not reach a spawned binary
/// from the developer's shell: `MINDTAPE_NOW` repins the clock every dated
/// write reads, `MINDTAPE_WORKTREE` retargets the project root, and `VISUAL`
/// would hand an accidental editor the developer's real one. `DIRENV_DIFF` and
/// `DIRENV_WATCHES` are direnv's record of what it loaded: inherited, they make
/// a `direnv exec` inside a scratch repo rewind `PATH` to the shell's pre-load
/// value, so a script resolves the host's `mt` instead of the one under test.
const CLEARED_VARS: [&str; 5] = [
    "MINDTAPE_NOW",
    "MINDTAPE_WORKTREE",
    "VISUAL",
    "DIRENV_DIFF",
    "DIRENV_WATCHES",
];

/// The editor every isolated spawn resolves instead of the developer's: a
/// command that cannot exist, so an `add` a test did not mean to make
/// interactive fails on the spawn rather than opening a real editor. Tests
/// that exercise the editor set their own `EDITOR` after [`isolate`].
const STUB_EDITOR: &str = "mt-no-such-editor";

/// The environment a spawned `mt` needs for a test to be about the code rather
/// than about the machine running it.
///
/// `mt` layers a global `defaults.toml` under every project config —
/// `dirs::config_dir()/mindtape/defaults.toml`, a supported and documented
/// layer (`crates/mindtape/src/config/layers.rs`). A spawned binary reads
/// whatever the developer happens to keep there, so without this every e2e
/// test is a function of their personal `mt` setup: two lines of `[ls]
/// hyperlinks` in `~/.config/mindtape/defaults.toml` are enough to fail
/// `hyperlinks_e2e` on a tree with no Rust change in it.
///
/// So point that lookup at an empty path. `MINDTAPE_CONFIG_HOME` is the one
/// knob that does it on every platform (`mindtape::config_home`): `dirs` reads
/// the environment only on Unix — `XDG_CONFIG_HOME` on Linux, `HOME` on macOS
/// (which reads `~/Library/Application Support` and ignores XDG) — and goes
/// through a known-folder API on Windows that no variable overrides. The XDG
/// pair stays set anyway, since `HOME` also caps the ancestor
/// `.mindtape/defaults.toml` walk at `dir`, so a stray one above the tempdir
/// cannot leak in either.
pub fn isolated_env(dir: &Path) -> [(&'static str, PathBuf); 3] {
    [
        (mindtape::CONFIG_HOME_ENV, dir.join(NO_GLOBAL_CONFIG)),
        ("XDG_CONFIG_HOME", dir.join(NO_GLOBAL_CONFIG)),
        ("HOME", dir.to_path_buf()),
    ]
}

/// `mt`, ready to run in `dir` with the developer's global config out of reach
/// ([`isolated_env`]). The spawn every e2e test goes through; a bare
/// `cargo_bin("mt")` is the un-isolated one and only `install_e2e` (which
/// injects its own XDG directories) has cause to write it.
pub fn mt(dir: &Path) -> assert_cmd::Command {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_mt"));
    command.current_dir(dir);
    isolate(&mut command, dir);
    assert_cmd::Command::from_std(command)
}

/// [`mt`]'s isolation for a caller that builds its own command — the pty tests
/// spawn `mt` through `script(1)`, which passes its environment down.
///
/// The editor pair is scrubbed here, not defaulted: a sweep mutant that
/// wrongly answers "open the editor" (`cmd::add::interactive::should_open -> true`
/// and friends) otherwise spawns whatever `$VISUAL`/`$EDITOR` the developer's
/// session carries. That happened: `cargo-mutants` ran the add e2e tests with
/// a `--wait` GUI editor in `VISUAL`, one window per test until the 60 s timeout. In a
/// tree whose suite must be about the code, the developer's editor can never
/// be the fallback; [`STUB_EDITOR`] makes the mistake a fast spawn error.
pub fn isolate<'a>(
    command: &'a mut std::process::Command,
    dir: &Path,
) -> &'a mut std::process::Command {
    for (key, value) in isolated_env(dir) {
        command.env(key, value);
    }
    for key in CLEARED_VARS {
        command.env_remove(key);
    }
    command.env("EDITOR", STUB_EDITOR);
    command
}

/// Every character that opens Typst syntax, in one line of otherwise ordinary
/// prose: emphasis (`*`, `_`), math (`$`), code (`#`), content-block delimiters
/// (`[`, `]`), a label (`<a>`) and a reference (`@at`). Text carrying these must
/// survive a write unchanged — unescaped, `create-*star*` fails to load and
/// `<a>`/`@at` silently evaluate away.
pub const MARKUP_HAZARDS: &str = "create-*star* _u_ $x$ #h [b] <a> >g @at";

/// A template whose slots span both sides of the escaping decision: a `string`
/// title and an `int` priority render as literals and must stay verbatim, while
/// the `text` slots land in markup and must be escaped. `why` sits inside a link
/// note, the one markup position the harvest reads back as a value, so a test can
/// assert what a filled `text` slot actually means rather than only how it looks.
const SLOT_TEMPLATE: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "Task title", example: "Fix the login redirect"),
  priority: slot("weight", kind: "int", default: 5),
  links: related("anchor.typ")[#slot("why", kind: "text", example: "same session store")],
)

== Summary

#slot("summary", kind: "text", default: "- TODO")
"#;

/// A throwaway project scaffolded with `source` as its default template
/// (`.mindtape/templates/task.typ`) — the shared prologue
/// [`project_with_slot_template`] and [`project_with_raw_slot_template`]
/// both build on before adding their own fixtures, and the one every test
/// about a template's *shape* (a single-line `task.with(…)`, a template that
/// cannot evaluate) reaches for directly.
pub fn project_with_template(source: &str) -> tempfile::TempDir {
    project_with_template_and_config("[tasks]\nroots = [\"tasks\"]\n", source)
}

/// [`project_with_template`], but with `config` written to
/// `.mindtape/config.toml` in place of the bare `[tasks] roots = ["tasks"]`
/// default — for a test whose default template must clear a rule the
/// project's own config declares (`[[tasks.body.headers.rules]]`,
/// `[[tasks.body.content.rules]]`).
pub fn project_with_template_and_config(config: &str, source: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    install_lib(dir.path());
    std::fs::create_dir_all(dir.path().join(".mindtape/templates")).unwrap();
    std::fs::write(dir.path().join(".mindtape/config.toml"), config).unwrap();
    std::fs::write(dir.path().join(".mindtape/templates/task.typ"), source).unwrap();
    dir
}

/// A throwaway project whose default template is [`SLOT_TEMPLATE`], with the
/// `anchor` task the template's `related` edge points at already filed.
pub fn project_with_slot_template() -> tempfile::TempDir {
    let dir = project_with_template(SLOT_TEMPLATE);
    std::fs::write(
        dir.path().join("tasks/anchor.typ"),
        "#import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(title: \"anchor\", status: proposed(2026, 7, 22))\n",
    )
    .unwrap();
    dir
}

/// A template with one `raw`-kind slot, the unescaped counterpart to
/// [`SLOT_TEMPLATE`]'s `text` slots — proves the escaping decision is a
/// per-slot choice the template author makes, not a per-command one.
const TYPST_SLOT_TEMPLATE: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "Task title", example: "Fix the login redirect"),
)

== Summary

#slot("summary", kind: "raw", default: "- `TODO`")
"#;

/// A throwaway project whose default template is [`TYPST_SLOT_TEMPLATE`].
pub fn project_with_raw_slot_template() -> tempfile::TempDir {
    project_with_template(TYPST_SLOT_TEMPLATE)
}

/// The canonical-JSON row `mt ls --json` reports for task `id` in `dir`. Reading
/// a value back through the harvest is the only way to tell text that survived a
/// write from text Typst quietly interpreted on the way in.
pub fn task_json(dir: &Path, id: &str) -> serde_json::Value {
    let output = mt(dir)
        .args(["ls", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    parsed
        .get("tasks")
        .and_then(serde_json::Value::as_array)
        .unwrap()
        .iter()
        .find(|task| task.get("id").and_then(serde_json::Value::as_str) == Some(id))
        .unwrap_or_else(|| panic!("no task {id:?} in {parsed}"))
        .clone()
}

/// The lone task's status note read back through the harvest, via an
/// explicit match-all filter (`mt ls "" --json`) since the default view hides
/// a task that has already reached a terminal status. Reading it back is the
/// only way to tell a note that survived the write from markup Typst quietly
/// interpreted away.
pub fn status_note(dir: &Path) -> String {
    let output = mt(dir)
        .args(["ls", "", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    parsed
        .pointer("/tasks/0/status/note")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("no status note in {parsed}"))
        .to_owned()
}

/// The value at RFC-6901 `pointer` inside a [`task_json`] row (`/links/0/note`).
/// Spelled as a lookup rather than `row["links"][0]`, which the workspace's
/// `indexing_slicing` lint denies.
pub fn at<'a>(row: &'a serde_json::Value, pointer: &str) -> &'a serde_json::Value {
    row.pointer(pointer)
        .unwrap_or_else(|| panic!("no {pointer} in {row}"))
}

/// Run `mt` in `dir` and assert it succeeds. Named `run_ok` (not `mt`) so it
/// sits alongside [`mt`] rather than shadowing it: callers that need the raw
/// `assert_cmd::Command` (a failure assertion, stdout capture, …) still reach
/// for [`mt`] directly.
pub fn run_ok(dir: &Path, args: &[&str]) {
    mt(dir).args(args).assert().success();
}

/// Run `mt` in `dir`, assert it succeeds, and return its stdout — [`run_ok`]
/// for the tests that assert on what was printed rather than only that it was.
pub fn stdout(dir: &Path, args: &[&str]) -> String {
    let out = mt(dir).args(args).output().unwrap();
    assert!(out.status.success(), "mt {args:?} failed: {out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// File a task in `dir` via `mt add --name <name> --set title=T <extra_set…>`,
/// then assert `mt ls` and `mt check` both succeed against it. Shared by the
/// `init` e2e batteries that prove a scaffolded preset or answer set is usable
/// end to end, not just that `init` wrote *something*.
pub fn add_then_ls_and_check(dir: &Path, name: &str, extra_set: &[&str]) {
    let mut args = vec!["add", "--name", name, "--set", "title=T"];
    args.extend_from_slice(extra_set);
    run_ok(dir, &args);
    run_ok(dir, &["ls"]);
    run_ok(dir, &["check"]);
}

/// A scaffolded project for `preset`, with the Typst package installed.
pub fn init_preset(preset: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    install_lib(dir.path());
    run_ok(dir.path(), &["init", "--preset", preset]);
    dir
}

/// Parsed `add --describe=json` schema rows run in `dir`.
pub fn describe_slots(dir: &Path) -> serde_json::Value {
    let output = mt(dir)
        .args(["add", "--describe=json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

/// Slot names from `add --describe=json` run in `dir`.
pub fn describe_slot_names(dir: &Path) -> Vec<String> {
    describe_slots(dir)
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row.get("name").and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

/// The `kind` reported for slot `name` in `add --describe=json` run in `dir`.
pub fn describe_slot_kind(dir: &Path, name: &str) -> String {
    describe_slots(dir)
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row.get("name").and_then(serde_json::Value::as_str) == Some(name))
        .and_then(|row| row.get("kind"))
        .and_then(serde_json::Value::as_str)
        .unwrap()
        .to_owned()
}

/// Write `config` to `.mindtape/config.toml` under an existing `dir`, install
/// the in-tree lib, and drop one task file per `(name, source)` pair into
/// `tasks/`. The `dir`-taking counterpart of [`scaffold_project`], for the
/// link-rewriting batteries (`rm_e2e`, `mv_e2e`) that need the temp directory
/// in hand before the project exists — a git repository is initialized over it.
pub fn scaffold_config(dir: &Path, config: &str, files: &[(&str, &str)]) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(dir.join(".mindtape/config.toml"), config).unwrap();
    install_lib(dir);
    for (name, contents) in files {
        std::fs::write(dir.join("tasks").join(name), contents).unwrap();
    }
}

/// Run `git` in `dir` and assert it succeeded.
pub fn git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

/// Turn `dir` into a committed-clean git repository: init, a local identity so
/// the commit does not depend on the developer's own, and one seed commit of
/// everything present. `commit.gpgsign=false` keeps a signing setup out of it.
pub fn git_commit_all(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "t"]);
    git(dir, &["add", "-A"]);
    git(
        dir,
        &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "seed"],
    );
}

/// Install the in-tree `@local/mindtape` lib override and an empty `tasks/`
/// root under `dir`, so a task filed there can actually be evaluated.
pub fn install_lib(dir: &Path) {
    mindtape::test_support::write_lib_package(dir, PRELUDE);
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
}

/// Scaffold a temp project that can evaluate `@local/mindtape` via the
/// in-tree lib override the loader honours, with the barebones `[tasks]
/// roots = ["tasks"]` config the `add` e2e batteries build on before
/// overriding it with a scenario-specific one.
pub fn scaffold(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(
        dir.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    install_lib(dir);
}

/// A throwaway project directory: the in-tree `@local/mindtape` package
/// override plus an empty `tasks/` root ([`install_lib`]), and `config` written
/// to `.mindtape/config.toml`. The shared prologue for every e2e test that
/// needs a real project with a config of its own.
pub fn scaffold_project(config: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    install_lib(dir.path());
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(dir.path().join(".mindtape/config.toml"), config).unwrap();
    dir
}

/// `[tasks] roots = ["tasks"]` — the minimal config both [`project_with_tasks`]
/// and [`project_with_task`] scaffold onto via [`scaffold_project`].
const TASKS_ROOT_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n";

/// A minimal real task source through the actual prelude: `title`, `wip`
/// since 2026-06-01, priority 3 — the plumbing [`project_with_tasks`] shares
/// across every fixture that needs several distinguishable real tasks.
fn task_source(title: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"{title}\",\n  status: wip(2026, 6, 1),\n  priority: 3,\n)\n"
    )
}

/// A throwaway project — [`scaffold_project`] with [`TASKS_ROOT_CONFIG`],
/// plus one real task per `(file, title)` pair, written under `tasks/`.
/// Unlike [`project_with_task`]'s single raw-metadata fixture, this goes
/// through the actual prelude and supports several tasks, for a test that
/// needs to tell them apart (`mt find`/`mt ls --format path` fixtures).
pub fn project_with_tasks(tasks: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = scaffold_project(TASKS_ROOT_CONFIG);
    for (file, title) in tasks {
        std::fs::write(dir.path().join("tasks").join(file), task_source(title)).unwrap();
    }
    dir
}

/// A throwaway project — [`scaffold_project`] with [`TASKS_ROOT_CONFIG`] and
/// one `wip` task titled `title` — for tests that only need something to list.
pub fn project_with_task(title: &str) -> tempfile::TempDir {
    let dir = scaffold_project(TASKS_ROOT_CONFIG);
    std::fs::write(
        dir.path().join("tasks/test.typ"),
        format!(
            r#"#metadata((
  marker: "mindtape.task",
  fields: (
    title: "{title}",
    status: (name: "wip", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 5,
  ),
  extras: (:),
))
"#
        ),
    )
    .unwrap();
    dir
}

/// A committed-clean repository holding one `wip` task, `parser`, with
/// `rules` appended to its config — where every `mt check --commit-msg` test
/// starts.
pub fn commit_repo(rules: &str) -> tempfile::TempDir {
    let dir = project_with_tasks(&[("parser.typ", "Write the parser")]);
    let config = dir.path().join(".mindtape/config.toml");
    let base = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, format!("{base}\n{rules}")).unwrap();
    git_commit_all(dir.path());
    dir
}

/// `mt check --commit-msg` over `message`, as a hook would run it: its
/// combined output, and whether it exited zero.
pub fn check_commit_msg(dir: &Path, message: &str) -> (bool, String) {
    let file = dir.join("COMMIT_MSG");
    std::fs::write(&file, message).unwrap();
    let out = mt(dir)
        .args(["check", "--commit-msg"])
        .arg(&file)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}
