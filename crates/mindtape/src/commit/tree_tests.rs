#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::{Path, PathBuf};

use super::{StateConfig, StateTree};
use crate::commit::tasks::{StatePlan, StateSource};
use crate::commit::tree_io::{parse_batch_bytes, read_blobs, repo_path};
use crate::config::{Config, parse_config};
use crate::test_support::commit_files;
use crate::test_support::git_tempdir;
#[cfg(unix)]
use crate::test_support::symlink_repo;
use crate::test_support::write_file;

/// A git repository holding `committed`, then each `staged` file written and
/// added to its index.
fn repo_with(committed: &[(&str, &str)], staged: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = git_tempdir();
    commit_files(dir.path(), committed, "seed");
    for (path, contents) in staged {
        write_file(dir.path(), path, contents);
        crate::test_support::git_as_tester(dir.path(), &["add", path]);
    }
    dir
}

/// [`parse_batch_bytes`] with each object read as text, for a test that only
/// cares about the contents.
fn parse_batch(out: &[u8], count: usize) -> Vec<Option<String>> {
    parse_batch_bytes(out, count)
        .into_iter()
        .map(|blob| blob.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
        .collect()
}

#[test]
fn a_package_path_resolves_against_the_repository_root() {
    let cases = [
        ("examples/rules/", "../../lib", Some("lib")),
        ("examples/rules/", "lib", Some("examples/rules/lib")),
        ("", "lib", Some("lib")),
        ("examples/rules/", "../../../lib", None),
        ("", "/lib", None),
    ];
    for (prefix, pkg, expected) in cases {
        assert_eq!(
            repo_path(prefix, Path::new(pkg)).as_deref(),
            expected,
            "{prefix:?} + {pkg:?}"
        );
    }
}

#[test]
fn a_batch_splits_into_objects_and_misses_in_order() {
    let out = b"abc blob 3\nfoo\nHEAD:gone.typ missing\ndef blob 0\n\n";
    assert_eq!(
        parse_batch(out, 3),
        [Some("foo".to_owned()), None, Some(String::new())]
    );
}

#[test]
fn a_short_batch_pads_with_misses() {
    assert_eq!(parse_batch(b"", 2), [None, None]);
    assert_eq!(parse_batch(b"HEAD:a b.typ missing\n", 1), [None]);
}

#[test]
fn a_batch_keeps_bytes_that_are_not_utf8() {
    let out = b"abc blob 2\n\xff\x00\n";
    assert_eq!(parse_batch_bytes(out, 1), [Some(vec![0xff, 0x00])]);
}

#[test]
fn blobs_read_each_revision_and_miss_what_it_lacks() {
    let dir = repo_with(&[("kept.typ", "old\n")], &[("kept.typ", "new\n")]);
    let root = dir.path();
    let specs = ["HEAD:kept.typ", ":kept.typ", "HEAD:gone.typ"].map(ToOwned::to_owned);
    assert_eq!(
        read_blobs(root, &specs).unwrap(),
        [Some("old\n".to_owned()), Some("new\n".to_owned()), None]
    );
}

/// A plan reading `rev` as the state at the project root.
fn plan(rev: &str) -> StatePlan {
    StatePlan {
        root_prefix: String::new(),
        head: StateSource {
            rev: rev.to_owned(),
            differs: Vec::new(),
        },
        staged: StateSource::default(),
        worktree: false,
    }
}

/// The config `rev` holds at the project root.
fn config_at(root: &Path, rev: &str) -> StateConfig {
    let state_plan = plan(rev);
    let tree = StateTree::new(root, &state_plan, &state_plan.head);
    let text = tree.config_text().unwrap().unwrap_or_default();
    tree.parse_config_in(&text, Some(root), None, None)
}

/// The task files `rev` holds under the roots `config` declares.
fn task_files_at(root: &Path, rev: &str, config: &Config) -> Vec<(String, PathBuf)> {
    let state_plan = plan(rev);
    let tree = StateTree::new(root, &state_plan, &state_plan.head);
    tree.task_files(config).unwrap()
}

/// A config with `roots` and one `mindtape` pin at `pin`.
fn pin_config(roots: &str, pin: &str) -> String {
    format!(
        "[tasks]\nroots = [\"{roots}\"]\n\n\
         [[typst.local-packages]]\nname = \"mindtape\"\npath = \"{pin}\"\n"
    )
}

#[test]
fn a_state_lists_every_task_file_under_its_roots_once() {
    let dir = git_tempdir();
    let root = dir.path();
    commit_files(
        root,
        &[
            ("tasks/a.typ", "a\n"),
            ("tasks/sub/b.typ", "b\n"),
            ("more/c.typ", "c\n"),
            ("notes.typ", "not a task\n"),
        ],
        "seed",
    );
    let config = parse_config("[tasks]\nroots = [\"tasks\", \"more\", \"tasks/sub\"]\n").unwrap();
    let files = task_files_at(root, "HEAD", &config);
    let rels: Vec<String> = files
        .iter()
        .map(|(_, rel)| crate::slash_path(rel))
        .collect();
    assert_eq!(rels, ["more/c.typ", "tasks/a.typ", "tasks/sub/b.typ"]);
}

#[test]
fn a_states_own_config_names_its_pins_and_roots() {
    let dir = git_tempdir();
    let root = dir.path();
    commit_files(
        root,
        &[(".mindtape/config.toml", &pin_config("old-tasks", "old-lib"))],
        "seed",
    );
    commit_files(
        root,
        &[(".mindtape/config.toml", &pin_config("tasks", "lib"))],
        "move",
    );

    let state = config_at(root, "HEAD~1");
    assert!(state.error.is_none());
    let config = state.config.unwrap();
    assert_eq!(config.tasks.roots[0].path(), "old-tasks");
    assert_eq!(config.typst.local_packages[0].path, "old-lib");
}

#[test]
fn a_missing_or_malformed_state_config_is_reported_not_read() {
    let dir = git_tempdir();
    let root = dir.path();
    commit_files(root, &[("tasks/a.typ", "a\n")], "no config");
    let missing = config_at(root, "HEAD");
    assert!(missing.config.is_some() && missing.error.is_none());

    commit_files(
        root,
        &[(".mindtape/config.toml", "[typst")],
        "broken config",
    );
    let broken = config_at(root, "HEAD");
    assert!(broken.config.is_none());
    let error = broken.error.unwrap();
    assert!(error.message.contains("config"), "{}", error.message);
}

#[test]
fn a_subproject_state_reads_its_prefix_config() {
    let dir = git_tempdir();
    let root = dir.path();
    commit_files(
        root,
        &[
            ("sub/.mindtape/config.toml", &pin_config("tasks", "lib")),
            ("sub/tasks/a.typ", "a\n"),
        ],
        "seed",
    );
    let mut state_plan = plan("HEAD");
    state_plan.root_prefix = "sub/".to_owned();
    let sub = root.join("sub");
    let tree = StateTree::new(&sub, &state_plan, &state_plan.head);
    let config = tree.config().unwrap().config.unwrap();
    assert_eq!(config.typst.local_packages[0].path, "lib");
    let files = tree.task_files(&config).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(crate::slash_path(&files[0].1), "tasks/a.typ");
}

#[test]
fn an_index_state_lists_the_task_files_the_index_holds() {
    let dir = repo_with(&[("tasks/a.typ", "a\n")], &[("tasks/b.typ", "b\n")]);
    let root = dir.path();
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\n").unwrap();
    let state_plan = plan("");
    let tree = StateTree::new(root, &state_plan, &state_plan.head);
    let files = tree.task_files(&config).unwrap();
    let rels: Vec<String> = files
        .iter()
        .map(|(_, rel)| crate::slash_path(rel))
        .collect();
    assert_eq!(rels, ["tasks/a.typ", "tasks/b.typ"]);
}

#[test]
fn a_states_packages_carry_the_versions_its_manifests_declare() {
    let dir = git_tempdir();
    let root = dir.path();
    let manifest =
        "[package]\nname = \"mindtape\"\nversion = \"0.2.0\"\nentrypoint = \"prelude.typ\"\n";
    commit_files(
        root,
        &[
            ("lib/typst.toml", manifest),
            ("lib/prelude.typ", "#let task(body, ..meta) = none\n"),
            (".mindtape/config.toml", &pin_config("tasks", "lib")),
        ],
        "seed",
    );
    let config = config_at(root, "HEAD").config.unwrap();
    let state_plan = plan("HEAD");
    let tree = StateTree::new(root, &state_plan, &state_plan.head);
    let packages = tree.packages(&config).unwrap().unwrap();
    assert_eq!(packages.specs[0].version.to_string(), "0.2.0");
}

/// A task entry for repository-relative `path` with `mode`.
fn entry(path: &str, mode: &str) -> super::TaskEntry {
    super::TaskEntry {
        repo: path.to_owned(),
        rel: PathBuf::from(path),
        oid: "0".to_owned(),
        mode: mode.to_owned(),
    }
}

/// The source the `rev` state resolves for `path` with `mode`.
fn source_at(root: &Path, rev: &str, path: &str, mode: &str) -> super::TaskSource {
    let state_plan = plan(rev);
    let tree = StateTree::new(root, &state_plan, &state_plan.head);
    tree.task_source(&entry(path, mode)).unwrap()
}

#[test]
fn a_missing_regular_task_blob_is_missing() {
    let dir = git_tempdir();
    let root = dir.path();
    commit_files(root, &[("seed.txt", "seed\n")], "seed");
    assert!(matches!(
        source_at(root, "HEAD", "tasks/gone.typ", "100644"),
        super::TaskSource::Missing
    ));
}

#[cfg(unix)]
#[test]
fn a_symlink_outside_the_repository_is_unsupported() {
    let dir = symlink_repo(&[("seed.txt", "seed\n")], &[("tasks/a.typ", "/etc/passwd")]);
    assert!(matches!(
        source_at(dir.path(), "HEAD", "tasks/a.typ", "120000"),
        super::TaskSource::Unsupported(_)
    ));
}

#[cfg(unix)]
#[test]
fn a_symlink_cycle_or_missing_target_is_unsupported() {
    let dir = symlink_repo(
        &[("seed.txt", "seed\n")],
        &[
            ("tasks/a.typ", "b.typ"),
            ("tasks/b.typ", "a.typ"),
            ("tasks/c.typ", "missing.typ"),
        ],
    );
    for path in ["tasks/a.typ", "tasks/c.typ"] {
        assert!(matches!(
            source_at(dir.path(), "HEAD", path, "120000"),
            super::TaskSource::Unsupported(_)
        ));
    }
}
