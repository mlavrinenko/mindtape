//! `mt @set ls` and `mt @set find`, end to end: one table over every project
//! of a set, grouped by project in set order, each under its own config.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use tempfile::TempDir;

use crate::support;

/// A set `@both` over `alpha` (two tasks, a `mine` view) and `beta` (one
/// task), plus the config home whose `at.toml` holds it.
struct Set {
    alpha: TempDir,
    beta: TempDir,
    cfg: TempDir,
}

impl Set {
    fn new() -> Self {
        let alpha = support::project_with_tasks(&[("one.typ", "One"), ("uno.typ", "Uno")]);
        let beta = support::project_with_tasks(&[("two.typ", "Two")]);
        let config = alpha.path().join(".mindtape/config.toml");
        let mut text = std::fs::read_to_string(&config).unwrap();
        text.push_str("\n[views.mine]\nfilter = 'contains(\"Uno\")'\n");
        std::fs::write(&config, text).unwrap();
        let cfg = tempfile::tempdir().unwrap();
        support::write_at_toml(cfg.path(), &[("both", &[alpha.path(), beta.path()])]);
        Self { alpha, beta, cfg }
    }

    /// A set whose `beta` has a config that does not parse.
    fn broken() -> Self {
        let set = Self::new();
        std::fs::write(set.beta.path().join(".mindtape/config.toml"), "[tasks\n").unwrap();
        set
    }

    fn alpha(&self) -> String {
        name(&self.alpha)
    }

    fn beta(&self) -> String {
        name(&self.beta)
    }

    /// `mt @both <args>` from an unrelated directory.
    fn run(&self, args: &[&str]) -> (Option<i32>, String, String) {
        let here = tempfile::tempdir().unwrap();
        let out = support::mt(here.path())
            .env(mindtape::CONFIG_HOME_ENV, self.cfg.path())
            .arg("@both")
            .args(args)
            .output()
            .unwrap();
        support::parts_of(&out)
    }

    /// [`Self::run`], which must exit `code`: its stdout.
    fn stdout(&self, args: &[&str], code: i32) -> String {
        let (got, stdout, stderr) = self.run(args);
        assert_eq!(got, Some(code), "{stderr}");
        stdout
    }

    /// [`Self::stdout`] of a run that exits 0, line by line.
    fn lines(&self, args: &[&str]) -> Vec<String> {
        self.stdout(args, 0).lines().map(str::to_owned).collect()
    }

    /// [`Self::run`], which must exit 1 naming `beta` on stderr: its stdout
    /// and stderr.
    fn failing_beta(&self, args: &[&str]) -> (String, String) {
        let (code, stdout, stderr) = self.run(args);
        assert_eq!(code, Some(1), "{stderr}");
        let note = format!("note: project: {}", self.beta());
        assert!(stderr.contains(&note), "{stderr}");
        (stdout, stderr)
    }
}

fn name(dir: &TempDir) -> String {
    dir.path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

/// The table rows (no header, no footer) as `(project, title)` pairs: the
/// first word, and the one before the captured table's `FILE` column.
fn rows(stdout: &str) -> Vec<(String, String)> {
    stdout
        .lines()
        .skip(1)
        .take_while(|line| !line.is_empty())
        .map(|line| {
            let words: Vec<&str> = line.split_whitespace().collect();
            let title = words.iter().rev().nth(1).unwrap();
            ((*words.first().unwrap()).to_owned(), (*title).to_owned())
        })
        .collect()
}

/// The project of each table row, in order.
fn projects(stdout: &str) -> Vec<String> {
    rows(stdout)
        .into_iter()
        .map(|(project, _)| project)
        .collect()
}

#[test]
fn a_set_lists_one_table_grouped_by_project_in_set_order() {
    let set = Set::new();
    let stdout = set.stdout(&["ls", ""], 0);
    assert!(stdout.starts_with("PROJECT "), "{stdout}");
    let expected = [set.alpha(), set.alpha(), set.beta()];
    assert_eq!(projects(&stdout), expected, "{stdout}");
    let footer = "\nyour filter · 3 of 3 tasks in 2 projects\n";
    assert!(stdout.ends_with(footer), "{stdout}");
}

#[test]
fn limit_counts_per_project() {
    let set = Set::new();
    let stdout = set.stdout(&["ls", "", "-n", "1"], 0);
    assert_eq!(projects(&stdout), [set.alpha(), set.beta()], "{stdout}");
    let footer = "your filter · 2 of 3 tasks in 2 projects\n\
                  1 more not shown · next page: mt @both ls '' -n 1 --skip 1\n";
    assert!(stdout.ends_with(footer), "{stdout}");
}

#[test]
fn a_project_lacking_the_view_fails_and_the_rest_still_list() {
    let set = Set::new();
    let (stdout, stderr) = set.failing_beta(&["ls", "@mine"]);
    let expected = [(set.alpha(), "Uno".to_owned())];
    assert_eq!(rows(&stdout), expected, "{stdout}");
    assert!(stderr.contains("unknown view: 'mine'"), "{stderr}");
    assert!(
        stderr.ends_with("error: 1 of 2 projects did not list\n"),
        "{stderr}"
    );
}

/// The table lists the rest and counts only them; the JSON keeps an object
/// for the broken member, holding its error.
#[test]
fn a_broken_member_exits_1_after_the_rest_print() {
    let set = Set::broken();
    let (table, _) = set.failing_beta(&["ls", ""]);
    assert_eq!(projects(&table), [set.alpha(), set.alpha()], "{table}");
    assert!(table.contains("· 2 of 2 tasks in 1 project\n"), "{table}");
    let (stdout, _) = set.failing_beta(&["ls", "", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let at = |pointer: &str| json.pointer(pointer).unwrap().clone();
    assert_eq!(json.as_array().unwrap().len(), 2, "{stdout}");
    assert_eq!(at("/0/project"), set.alpha().as_str());
    assert_eq!(at("/0/tasks").as_array().unwrap().len(), 2);
    assert!(at("/0/applied").is_object(), "{stdout}");
    assert!(at("/0/error").is_null(), "{stdout}");
    assert_eq!(at("/1/project"), set.beta().as_str());
    assert!(at("/1/error").is_string(), "{stdout}");
    assert_eq!(at("/1/tasks"), serde_json::json!([]));
}

#[test]
fn csv_gains_a_project_column() {
    let set = Set::new();
    let lines = set.lines(&["ls", "", "--format", "csv"]);
    assert_eq!(lines.len(), 4, "{lines:?}");
    assert!(lines.first().unwrap().starts_with("project,"), "{lines:?}");
    let beta = format!("{},", set.beta());
    assert!(lines.last().unwrap().starts_with(&beta), "{lines:?}");
}

#[test]
fn path_lists_every_project_in_order() {
    let set = Set::new();
    let lines = set.lines(&["ls", "", "--format", "path"]);
    let ends: Vec<bool> = lines.iter().map(|line| line.ends_with(".typ")).collect();
    assert_eq!(ends, [true; 3], "{lines:?}");
    assert!(lines.last().unwrap().ends_with("two.typ"), "{lines:?}");
    let stdout = set.stdout(&["ls", "", "--format", "path0"], 0);
    assert_eq!(stdout.matches('\0').count(), 3, "{stdout:?}");
}

#[test]
fn columns_name_the_columns_after_the_project() {
    let set = Set::new();
    let lines = set.lines(&["ls", "", "--columns", "title"]);
    let header = lines.first().unwrap().split_whitespace();
    assert_eq!(header.collect::<Vec<_>>(), ["PROJECT", "TITLE"]);
}

#[test]
fn find_lists_a_set_the_same_way() {
    let set = Set::new();
    let stdout = set.stdout(&["find", "Two"], 0);
    assert_eq!(rows(&stdout), [(set.beta(), "Two".to_owned())], "{stdout}");
}

#[test]
fn interactive_refuses_a_set() {
    let set = Set::new();
    let (code, _, stderr) = set.run(&["ls", "-i"]);
    assert_eq!(code, Some(2), "{stderr}");
    assert!(
        stderr.starts_with("error: --interactive lists one project\n"),
        "{stderr}"
    );
    let hint = format!(
        "hint: name one project, as in mt @{} ls --interactive\n",
        set.alpha()
    );
    assert!(stderr.contains(&hint), "{stderr}");
}
