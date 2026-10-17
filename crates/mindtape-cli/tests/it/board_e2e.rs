//! `mt board` and a bare `mt @name`, end to end: one row per project, a hidden
//! row, a broken member, a failed cell and `--json`.
#![allow(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing
)]

use std::path::Path;

use tempfile::TempDir;

use crate::support;

/// The projects of `@all` and the config home naming them.
struct Fixture {
    /// One task.
    alpha: TempDir,
    /// Two tasks.
    beta: TempDir,
    /// No task, so the shipped `hide` hides it.
    empty: TempDir,
    /// A config that does not parse.
    broken: TempDir,
    cfg: TempDir,
}

fn name(dir: &TempDir) -> String {
    dir.path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn toml_path(dir: &Path) -> String {
    dir.display().to_string().replace('\\', "/")
}

/// `@all` holds alpha, beta and empty; `@bad` adds the broken one; `@alpha`
/// is alpha alone. `extra` goes at the end of `at.toml`.
fn setup(extra: &str) -> Fixture {
    let alpha = support::project_with_tasks(&[("one.typ", "One")]);
    let beta = support::project_with_tasks(&[("two.typ", "Two"), ("three.typ", "Three")]);
    let empty = support::project_with_tasks(&[]);
    let broken = support::project_with_tasks(&[]);
    std::fs::write(broken.path().join(".mindtape/config.toml"), "[tasks\n").unwrap();
    let cfg = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(cfg.path().join("mindtape")).unwrap();
    let [one, two, none, bad] = [&alpha, &beta, &empty, &broken].map(|dir| toml_path(dir.path()));
    std::fs::write(
        cfg.path().join("mindtape/at.toml"),
        format!(
            "[at.alpha]\ninclude = [\"{one}\"]\n\n\
             [at.all]\ninclude = [\"{one}\", \"{two}\", \"{none}\"]\n\n\
             [at.bad]\ninclude = [\"{one}\", \"{bad}\"]\n\n{extra}"
        ),
    )
    .unwrap();
    Fixture {
        alpha,
        beta,
        empty,
        broken,
        cfg,
    }
}

/// `mt <args>` in `dir` under the fixture's config home: its exit code,
/// stdout and stderr.
fn mt_in(fx: &Fixture, dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = support::mt(dir)
        .env(mindtape::CONFIG_HOME_ENV, fx.cfg.path())
        .args(args)
        .output()
        .unwrap();
    let text = |bytes: Vec<u8>| String::from_utf8(bytes).unwrap();
    (out.status.code(), text(out.stdout), text(out.stderr))
}

/// `mt <args>` outside any project, with its exit code checked: its stdout
/// and stderr.
fn board(fx: &Fixture, args: &[&str], code: i32) -> (String, String) {
    let here = tempfile::tempdir().unwrap();
    let (got, stdout, stderr) = mt_in(fx, here.path(), args);
    assert_eq!(got, Some(code), "{args:?}: {stdout}{stderr}");
    (stdout, stderr)
}

/// The words of line `n` of `text`.
fn words(text: &str, n: usize) -> Vec<&str> {
    text.lines()
        .nth(n)
        .unwrap_or("")
        .split_whitespace()
        .collect()
}

/// The first word of each line of the table, header and footer excluded.
fn row_names(text: &str) -> Vec<String> {
    text.lines()
        .skip(1)
        .take_while(|line| !line.is_empty())
        .map(|line| line.split_whitespace().next().unwrap_or("").to_owned())
        .collect()
}

#[test]
fn a_set_prints_one_row_per_project_sorted_with_the_empty_one_hidden() {
    let fx = setup("");
    let (text, _) = board(&fx, &["@all", "board"], 0);
    let header: Vec<&str> = text.lines().next().unwrap().split_whitespace().collect();
    assert_eq!(header, ["PROJECT", "OPEN", "READY", "BLOCKED", "NEXT"]);
    assert_eq!(
        row_names(&text),
        [name(&fx.beta), name(&fx.alpha)],
        "{text}"
    );
    let alpha = words(&text, 2);
    assert_eq!(
        (&alpha[1..4], alpha.last()),
        (&["1", "1", "0"][..], Some(&"One"))
    );
    let footer = "@all · 3 projects · 1 hidden · mt @all board --all";
    assert_eq!(text.lines().rev().take(2).collect::<Vec<_>>(), [footer, ""]);
}

#[test]
fn all_shows_the_hidden_row() {
    let fx = setup("");
    let (text, _) = board(&fx, &["@all", "board", "--all"], 0);
    let names = [name(&fx.beta), name(&fx.alpha), name(&fx.empty)];
    assert_eq!(row_names(&text), names, "{text}");
    assert_eq!(text.lines().last(), Some("@all · 3 projects"));
}

#[test]
fn a_bare_name_runs_the_board_and_takes_its_flags() {
    let fx = setup("");
    assert_eq!(board(&fx, &["@all"], 0), board(&fx, &["@all", "board"], 0));
    let (text, _) = board(&fx, &["@all", "--all"], 0);
    assert!(text.contains(&name(&fx.empty)), "{text}");
    let (text, _) = board(&fx, &["@alpha"], 0);
    assert_eq!(row_names(&text), [name(&fx.alpha)], "{text}");
}

#[test]
fn the_current_project_is_one_row() {
    let fx = setup("");
    let (code, text, _) = mt_in(&fx, fx.beta.path(), &["board"]);
    assert_eq!(code, Some(0), "{text}");
    assert_eq!(row_names(&text), [name(&fx.beta)], "{text}");
}

#[test]
fn a_broken_member_is_a_warning_row_and_exits_1_after_the_others_print() {
    let fx = setup("");
    let (text, stderr) = board(&fx, &["@bad"], 1);
    let broken = format!("⚠ {}", name(&fx.broken));
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[1].starts_with(&name(&fx.alpha)), "{text}");
    assert!(
        lines[2].starts_with(&broken) && lines[2].contains("config.toml"),
        "{text}"
    );
    assert!(
        stderr.contains("error: 1 project failed to load"),
        "{stderr}"
    );
}

#[test]
fn a_failed_cell_prints_a_question_mark_and_a_note_naming_its_project() {
    let fx = setup(
        "[[board.columns]]\nname = \"odd\"\nfilter = 'status == \"no-such-status\"'\n\n\
         [[board.columns]]\nname = \"open\"\nfilter = \"!terminal\"\n",
    );
    let (text, _) = board(&fx, &["@alpha", "board"], 0);
    assert_eq!(words(&text, 1), [name(&fx.alpha).as_str(), "?", "1"]);
    let note = format!("note: {}: column 'odd': ", name(&fx.alpha));
    assert!(text.contains(&note), "{text}");
}

#[test]
fn json_prints_every_row_with_its_cells() {
    let fx = setup("");
    let (text, _) = board(&fx, &["@bad", "board", "--json"], 1);
    let rows: serde_json::Value = serde_json::from_str(&text).unwrap();
    let alpha = &rows[0];
    assert_eq!(alpha["project"], name(&fx.alpha));
    assert_eq!(alpha["hidden"], false);
    assert_eq!(alpha["cells"]["open"], 1);
    assert_eq!(alpha["cells"]["next"]["id"], "one");
    assert_eq!(alpha["cells"]["next"]["title"], "One");
    let broken = &rows[1];
    assert_eq!(broken["project"], name(&fx.broken));
    let error = broken["error"].as_str().unwrap();
    assert!(error.contains("config.toml"), "{error}");
    assert_eq!(rows.as_array().map(Vec::len), Some(2));
}
