//! Unit tests for `at.toml`: parsing, `@name` resolution, globs, `exclude`,
//! `forks` and member naming. A [`FakeFs`] stands in for the disk, so these
//! touch no real paths.

#![allow(clippy::unwrap_used)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::{AtConfig, AtError, AtFs, Forks, Target};

/// An in-memory tree: the directories that exist and the files' text.
#[derive(Default)]
struct FakeFs {
    dirs: BTreeSet<PathBuf>,
    /// Directories that can be entered but not listed, as with mode `111`.
    unlisted: BTreeSet<PathBuf>,
    files: BTreeMap<PathBuf, String>,
}

impl FakeFs {
    /// A project at `path`, with every ancestor directory.
    fn project(mut self, path: &str) -> Self {
        self.dir(path);
        self.dir(&format!("{path}/.mindtape"));
        self
    }

    /// A project at `path` whose `.git` points at `gitdir`; `linked` adds the
    /// `commondir` a linked worktree's gitdir holds.
    fn git_project(mut self, path: &str, gitdir: &str, linked: bool) -> Self {
        self = self.project(path);
        self.files.insert(
            PathBuf::from(format!("{path}/.git")),
            format!("gitdir: {gitdir}\n"),
        );
        if linked {
            let gitdir = Path::new(path).join(gitdir);
            self.files
                .insert(gitdir.join("commondir"), "../..\n".to_owned());
        }
        self
    }

    /// A plain directory with no `.mindtape/`.
    fn plain(mut self, path: &str) -> Self {
        self.dir(path);
        self
    }

    fn dir(&mut self, path: &str) {
        let mut path = PathBuf::from(path);
        loop {
            self.dirs.insert(path.clone());
            if !path.pop() || path.as_os_str().is_empty() {
                break;
            }
        }
    }
}

impl AtFs for FakeFs {
    fn subdirs(&self, dir: &Path) -> Vec<String> {
        if self.unlisted.contains(dir) {
            return Vec::new();
        }
        self.dirs
            .iter()
            .filter(|d| d.parent() == Some(dir))
            .filter_map(|d| d.file_name()?.to_str().map(str::to_owned))
            .collect()
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.contains(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    fn read_file(&self, path: &Path) -> Option<String> {
        self.files.get(path).cloned()
    }
}

fn home() -> &'static Path {
    Path::new("/h")
}

fn resolve(toml: &str, fs: &FakeFs, token: &str) -> Result<Option<Target>, AtError> {
    AtConfig::parse(toml)
        .unwrap()
        .resolve(token, Some(home()), fs)
}

/// The member names and paths of the set `token` resolves to.
fn members(toml: &str, fs: &FakeFs, token: &str) -> Vec<(String, PathBuf)> {
    match resolve(toml, fs, token).unwrap() {
        Some(Target::Set(members)) => members.into_iter().map(|m| (m.name, m.path)).collect(),
        other => panic!("expected a set, got {other:?}"),
    }
}

/// Assert the set `token` resolves to holds `want`, as name and path pairs.
fn assert_members(toml: &str, fs: &FakeFs, token: &str, want: &[(&str, &str)]) {
    let want: Vec<(String, PathBuf)> = want
        .iter()
        .map(|(name, path)| ((*name).to_owned(), PathBuf::from(path)))
        .collect();
    assert_eq!(members(toml, fs, token), want);
}

/// The project path `token` resolves to, or `None` when nothing answers.
fn project(toml: &str, fs: &FakeFs, token: &str) -> Option<PathBuf> {
    match resolve(toml, fs, token).unwrap() {
        Some(Target::Project(path)) => Some(path),
        None => None,
        other => panic!("expected a project, got {other:?}"),
    }
}

/// The member names and fork flags of a resolved set.
fn fork_flags(target: Option<Target>) -> Vec<(String, bool)> {
    let Some(Target::Set(set)) = target else {
        panic!("expected a set, got {target:?}");
    };
    set.into_iter().map(|m| (m.name, m.fork)).collect()
}

/// `want` as owned name and fork-flag pairs.
fn flags(want: &[(&str, bool)]) -> Vec<(String, bool)> {
    want.iter()
        .map(|(name, fork)| ((*name).to_owned(), *fork))
        .collect()
}

const ABG: &[(&str, &str)] = &[
    ("alpha", "/h/p/alpha"),
    ("beta", "/h/p/beta"),
    ("gamma", "/h/p/gamma"),
];

fn home_fs() -> FakeFs {
    FakeFs::default()
        .project("/h/p/alpha")
        .project("/h/p/beta")
        .plain("/h/p/notes")
        .git_project(
            "/h/p/alpha-fork",
            "../alpha/.git/worktrees/alpha-fork",
            true,
        )
        .git_project("/h/p/gamma", "../gamma-mod/.git/modules/gamma", false)
}

const HOME: &str = r#"
[at.home]
include = ["~/p/*"]
"#;

#[test]
fn parses_entries_with_defaults() {
    let config = AtConfig::parse(
        r#"
        [at.mt]
        include = ["~/p/mindtape"]

        [at.home]
        include = ["~/p/*", "@mt"]
        exclude = ["~/p/old"]
        forks = "only"
        "#,
    )
    .unwrap();
    assert_eq!(config.names(), vec!["home", "mt"]);
    let mt = config.entry("mt").unwrap();
    assert_eq!(mt.include, vec!["~/p/mindtape"]);
    assert!(mt.exclude.is_empty());
    assert_eq!(mt.forks, Forks::Skip);
    let home = config.entry("home").unwrap();
    assert_eq!(home.exclude, vec!["~/p/old"]);
    assert_eq!(home.forks, Forks::Only);
}

#[test]
fn forks_takes_a_bool_or_only() {
    for (text, want) in [
        ("false", Forks::Skip),
        ("true", Forks::Keep),
        ("\"only\"", Forks::Only),
    ] {
        let config =
            AtConfig::parse(&format!("[at.x]\ninclude = [\"~/a/*\"]\nforks = {text}\n")).unwrap();
        assert_eq!(config.entry("x").unwrap().forks, want, "forks = {text}");
    }
    let err = AtConfig::parse("[at.x]\ninclude = [\"~/a/*\"]\nforks = \"all\"\n").unwrap_err();
    assert!(matches!(err, AtError::Parse(_)), "got {err:?}");
}

#[test]
fn unknown_keys_are_refused() {
    for text in ["bogus = 1", "[at.x]\ninclude = [\"~/a\"]\npath = \"~/b\"\n"] {
        let err = AtConfig::parse(text).unwrap_err();
        assert!(matches!(err, AtError::Parse(_)), "{text}: got {err:?}");
    }
}

#[test]
fn an_entry_without_include_is_refused() {
    let err = AtConfig::parse("[at.x]\nexclude = [\"~/a\"]\n").unwrap_err();
    assert!(
        matches!(&err, AtError::Entry { name, .. } if name == "x"),
        "got {err:?}"
    );
}

#[test]
fn an_entry_name_must_be_one_path_segment() {
    let err = AtConfig::parse("[at.\"a/b\"]\ninclude = [\"~/a\"]\n").unwrap_err();
    assert!(matches!(err, AtError::InvalidName { .. }), "got {err:?}");
}

#[test]
fn an_invalid_token_is_refused() {
    for token in ["", ".", "..", "a\\b", "/a", "a/", "a//b", "a/../b"] {
        let err = resolve(HOME, &home_fs(), token).unwrap_err();
        assert!(
            matches!(err, AtError::InvalidName { .. }),
            "{token:?}: got {err:?}"
        );
    }
}

#[test]
fn one_literal_path_is_a_project() {
    let fs = FakeFs::default();
    assert_eq!(
        project("[at.mt]\ninclude = [\"~/p/mindtape\"]\n", &fs, "mt"),
        Some(PathBuf::from("/h/p/mindtape"))
    );
}

#[test]
fn a_glob_matching_one_project_is_still_a_set() {
    let fs = FakeFs::default().project("/h/solo/only");
    assert_members(
        "[at.s]\ninclude = [\"~/solo/*\"]\n",
        &fs,
        "s",
        &[("only", "/h/solo/only")],
    );
}

#[test]
fn a_glob_keeps_projects_and_skips_forks_by_default() {
    assert_members(HOME, &home_fs(), "home", ABG);
}

#[test]
fn forks_true_keeps_forks_and_marks_them() {
    let toml = "[at.home]\ninclude = [\"~/p/*\"]\nforks = true\n";
    assert_eq!(
        fork_flags(resolve(toml, &home_fs(), "home").unwrap()),
        flags(&[
            ("alpha", false),
            ("alpha-fork", true),
            ("beta", false),
            ("gamma", false)
        ])
    );
}

#[test]
fn forks_only_keeps_only_forks() {
    let toml = "[at.home]\ninclude = [\"~/p/*\"]\nforks = \"only\"\n";
    assert_members(
        toml,
        &home_fs(),
        "home",
        &[("alpha-fork", "/h/p/alpha-fork")],
    );
}

#[test]
fn exclude_drops_paths_globs_and_names() {
    let toml = r#"
        [at.home]
        include = ["~/p/*", "~/x/*"]
        exclude = ["~/p/beta", "~/x/*", "@g"]

        [at.g]
        include = ["~/p/gamma"]
    "#;
    let fs = home_fs().project("/h/x/one");
    assert_members(toml, &fs, "home", &[("alpha", "/h/p/alpha")]);
}

#[test]
fn a_set_reads_one_level_per_glob_and_joins_at_names() {
    let fs = home_fs()
        .project("/h/w/a/repo")
        .project("/h/w/b/repo")
        .project("/h/w/a/deep/repo")
        .project("/h/q/delta");
    let toml = r#"
        [at.w]
        include = ["~/w/*/repo"]

        [at.both]
        include = ["@one", "@two", "~/p/alpha"]

        [at.one]
        include = ["~/p/alpha"]

        [at.two]
        include = ["~/q/*"]
    "#;
    let cases: [(&str, &[(&str, &str)]); 2] = [
        ("w", &[("a/repo", "/h/w/a/repo"), ("b/repo", "/h/w/b/repo")]),
        ("both", &[("alpha", "/h/p/alpha"), ("delta", "/h/q/delta")]),
    ];
    for (token, want) in cases {
        assert_members(toml, &fs, token, want);
    }
}

#[test]
fn an_at_name_include_can_name_a_directory() {
    let toml = r#"
        [at.pick]
        include = ["@beta", "@alpha"]

        [at.home]
        include = ["~/p/*"]
    "#;
    assert_members(
        toml,
        &home_fs(),
        "pick",
        &[("beta", "/h/p/beta"), ("alpha", "/h/p/alpha")],
    );
}

#[test]
fn an_unknown_at_name_include_is_refused() {
    let err = resolve("[at.x]\ninclude = [\"@nope\"]\n", &home_fs(), "x").unwrap_err();
    assert!(
        matches!(&err, AtError::Unknown { name } if name == "nope"),
        "got {err:?}"
    );
}

#[test]
fn a_cycle_is_refused_naming_its_chain() {
    let toml = r#"
        [at.a]
        include = ["@b"]

        [at.b]
        include = ["~/p/alpha", "@a"]
    "#;
    let err = resolve(toml, &home_fs(), "a").unwrap_err();
    match err {
        AtError::Cycle { chain } => assert_eq!(chain, vec!["a", "b", "a"]),
        other => panic!("expected a cycle, got {other:?}"),
    }
}

#[test]
fn a_name_resolves_to_one_directory_or_nothing() {
    let home = "[at.home]\ninclude = [\"~/p/*\"]\nexclude = [\"~/p/beta\"]\n";
    let mt = "[at.mt]\ninclude = [\"~/p/mindtape\"]\n";
    let wins = "[at.alpha]\ninclude = [\"~/p/beta\"]\n\n[at.home]\ninclude = [\"~/p/*\"]\n";
    let twice = "[at.a]\ninclude = [\"~/p/*\"]\n\n[at.b]\ninclude = [\"~/p/alpha\"]\n";
    let cases = [
        // forks and exclude don't hide a directory from its name
        (home, "alpha-fork", Some("/h/p/alpha-fork")),
        (home, "beta", Some("/h/p/beta")),
        // a literal include answers to its directory name
        (mt, "mindtape", Some("/h/p/mindtape")),
        // an entry name wins over a directory
        (wins, "alpha", Some("/h/p/beta")),
        // one directory reached twice is no clash
        (twice, "alpha", Some("/h/p/alpha")),
        // a glob skips a directory with no .mindtape/
        (HOME, "notes", None),
        (HOME, "zeta", None),
    ];
    for (toml, token, want) in cases {
        let got = project(toml, &home_fs(), token);
        assert_eq!(got, want.map(PathBuf::from), "@{token} in {toml}");
    }
}

#[test]
fn two_directories_with_one_name_are_refused_naming_both() {
    let toml = "[at.a]\ninclude = [\"~/a/*\"]\n\n[at.b]\ninclude = [\"~/b/repo\"]\n";
    let fs = FakeFs::default().project("/h/a/repo");
    let err = resolve(toml, &fs, "repo").unwrap_err();
    match err {
        AtError::Clash { name, candidates } => {
            assert_eq!(name, "repo");
            let named: Vec<_> = candidates
                .iter()
                .map(|member| (member.name.as_str(), member.path.clone()))
                .collect();
            assert_eq!(
                named,
                vec![
                    ("a/repo", PathBuf::from("/h/a/repo")),
                    ("b/repo", PathBuf::from("/h/b/repo"))
                ]
            );
        }
        other => panic!("expected a clash, got {other:?}"),
    }
}

#[test]
fn a_path_suffix_picks_one_of_the_clashing_directories() {
    let toml = "[at.a]\ninclude = [\"~/a/*\"]\n\n[at.b]\ninclude = [\"~/b/repo\"]\n";
    let fs = FakeFs::default().project("/h/a/repo");
    for (token, want) in [
        ("a/repo", Some("/h/a/repo")),
        ("h/b/repo", Some("/h/b/repo")),
        ("c/repo", None),
    ] {
        assert_eq!(
            project(toml, &fs, token),
            want.map(PathBuf::from),
            "@{token}"
        );
    }
}

#[test]
fn a_member_reached_twice_is_listed_once() {
    let toml = "[at.s]\ninclude = [\"~/p/alpha\", \"~/p/*\"]\n";
    assert_members(toml, &home_fs(), "s", ABG);
}

#[test]
fn clashing_members_take_their_shortest_unique_suffix() {
    let fs = FakeFs::default()
        .project("/h/a/x/repo")
        .project("/h/b/x/repo")
        .project("/h/c/repo")
        .project("/h/c/solo");
    let toml = "[at.s]\ninclude = [\"~/a/x/repo\", \"~/b/x/repo\", \"~/c/*\"]\n";
    assert_members(
        toml,
        &fs,
        "s",
        &[
            ("a/x/repo", "/h/a/x/repo"),
            ("b/x/repo", "/h/b/x/repo"),
            ("c/repo", "/h/c/repo"),
            ("solo", "/h/c/solo"),
        ],
    );
}

#[test]
fn a_path_without_home_keeps_its_tilde() {
    let config = AtConfig::parse("[at.mt]\ninclude = [\"~/p/mindtape\"]\n").unwrap();
    let got = config.resolve("mt", None, &FakeFs::default()).unwrap();
    assert_eq!(got, Some(Target::Project(PathBuf::from("~/p/mindtape"))));
}

#[test]
fn a_literal_segment_is_entered_without_listing_its_parent() {
    let mut fs = home_fs();
    fs.unlisted.insert(PathBuf::from("/h"));
    assert_members(HOME, &fs, "home", ABG);
}

#[test]
fn a_bad_glob_is_refused() {
    let err = resolve("[at.s]\ninclude = [\"~/p/[a\"]\n", &home_fs(), "s").unwrap_err();
    assert!(matches!(err, AtError::Glob { .. }), "got {err:?}");
}

#[test]
fn a_gitdir_file_without_its_prefix_is_no_fork() {
    let mut fs = FakeFs::default().project("/h/p/odd");
    fs.files
        .insert(PathBuf::from("/h/p/odd/.git"), "nonsense\n".to_owned());
    let toml = "[at.s]\ninclude = [\"~/p/*\"]\nforks = \"only\"\n";
    assert_eq!(members(toml, &fs, "s"), vec![]);
}
