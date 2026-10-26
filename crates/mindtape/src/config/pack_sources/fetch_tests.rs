//! The fetch against a local bare repository: one with a file a checkout
//! would turn to CRLF, one with a symlink, and two fetches at once.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::ffi::OsStr;
use std::path::Path;

use super::{Fetch, FetchError, fetch, locate};
use crate::config::pack_sources::git::git_error;
use crate::test_support::{git_as_tester, git_stdout, rerun_isolated_with, tempdir, write_file};

/// The framed hash of [`source`]'s tree, the same on every platform.
const SOURCE_HASH: &str = "32c8ada3417d4cd0d8ec52f8487b73232601df2a20f9992d6aadfbd3eac3d386";

/// [`fetch`] through `program` in place of `git`, with no expected hash.
fn fetch_with(
    program: &OsStr,
    git: &str,
    reference: &str,
    commit: &str,
    cache_root: &Path,
) -> Result<String, FetchError> {
    let request = Fetch {
        program,
        git,
        reference,
        commit,
        expected: None,
    };
    request.run(cache_root)
}

/// A tempdir holding an empty git repository `work`, made by `init`.
fn work_repo(init: &[&str]) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempdir();
    let work = dir.path().join("work");
    std::fs::create_dir(&work).unwrap();
    git_as_tester(&work, init);
    (dir, work)
}

/// Tag `commit` of `work` as `v1`, clone `work` bare to `src.git` beside
/// it, and return the commit.
fn publish(dir: &Path, work: &Path, commit: &str) -> String {
    git_as_tester(work, &["tag", "v1", commit]);
    git_as_tester(dir, &["clone", "--quiet", "--bare", "work", "src.git"]);
    git_stdout(work, &["rev-parse", "v1^{commit}"])
}

/// A bare repository holding one commit, tagged `v1` and on `main`, and that
/// commit. `.gitattributes` asks a checkout for CRLF line ends, which the
/// fetch must not apply, and `link` is a symlink when `with_symlink`.
fn source(with_symlink: bool) -> (tempfile::TempDir, String) {
    let (dir, work) = work_repo(&["init", "--quiet"]);
    write_file(&work, ".gitattributes", "*.toml text eol=crlf\n");
    write_file(&work, "commits.toml", "a = 1\nb = 2\n");
    write_file(&work, "team/pack.toml", "c = 3\n");
    git_as_tester(&work, &["add", "."]);
    if with_symlink {
        let blob = git_stdout(&work, &["hash-object", "-w", "commits.toml"]);
        let info = format!("120000,{blob},link");
        git_as_tester(&work, &["update-index", "--add", "--cacheinfo", &info]);
    }
    git_as_tester(&work, &["commit", "--quiet", "-m", "packs"]);
    git_as_tester(&work, &["branch", "-M", "main"]);
    let commit = publish(dir.path(), &work, "HEAD");
    (dir, commit)
}

/// A [`source`] repository and an empty cache to fetch it into.
struct Fixture {
    src: tempfile::TempDir,
    commit: String,
    cache: tempfile::TempDir,
}

impl Fixture {
    fn new(with_symlink: bool) -> Self {
        let (src, commit) = source(with_symlink);
        Self {
            src,
            commit,
            cache: tempdir(),
        }
    }

    fn url(&self) -> String {
        self.src.path().join("src.git").display().to_string()
    }

    /// [`fetch`] of `reference` at the fixture's commit.
    fn fetch(&self, reference: &str, expected: Option<&str>) -> Result<String, FetchError> {
        fetch(
            &self.url(),
            reference,
            &self.commit,
            expected,
            self.cache.path(),
        )
    }

    /// How many entries the cache root holds.
    fn cached(&self) -> usize {
        std::fs::read_dir(self.cache.path()).unwrap().count()
    }
}

#[test]
fn a_fetch_lands_the_commit_byte_for_byte_under_its_hash() {
    let fixture = Fixture::new(false);
    assert_eq!(fixture.fetch("v1", None).unwrap(), SOURCE_HASH);
    let tree = fixture.cache.path().join(SOURCE_HASH);
    let read = |path: &str| std::fs::read(tree.join(path)).unwrap();
    assert_eq!(read("commits.toml"), b"a = 1\nb = 2\n");
    assert_eq!(read("team/pack.toml"), b"c = 3\n");
    assert!(!tree.join(".git").exists());
    assert_eq!(fixture.cached(), 1, "the temporary directory is gone");
}

#[test]
fn a_branch_ref_fetches_the_same_tree() {
    let fixture = Fixture::new(false);
    let sha = fixture.fetch("main", Some(SOURCE_HASH)).unwrap();
    assert_eq!(sha, SOURCE_HASH);
}

#[test]
fn an_expected_hash_that_differs_fails_naming_both() {
    let fixture = Fixture::new(false);
    let expected = "0".repeat(64);
    let err = fixture.fetch("v1", Some(&expected)).unwrap_err();
    let shown = err.to_string();
    assert!(
        shown.contains(&expected) && shown.contains(SOURCE_HASH),
        "{shown}"
    );
    assert!(matches!(err, FetchError::Mismatch { .. }), "{err:?}");
    assert_eq!(fixture.cached(), 0);
}

#[test]
fn a_symlink_in_the_source_fails() {
    let err = Fixture::new(true).fetch("v1", None).unwrap_err();
    assert_eq!(err.to_string(), "pack source holds a symlink: 'link'");
}

#[test]
fn a_commit_the_ref_lacks_fails() {
    let mut fixture = Fixture::new(false);
    fixture.commit = "1".repeat(40);
    let err = fixture.fetch("v1", None).unwrap_err();
    assert!(matches!(err, FetchError::MissingCommit { .. }), "{err:?}");
    assert!(err.hint().is_some());
}

#[test]
fn a_commit_that_is_not_hex_fails_before_git_runs() {
    let cache = tempdir();
    let err = fetch_with(
        OsStr::new("mt-no-such-git"),
        "x",
        "v1",
        "--all",
        cache.path(),
    );
    assert!(matches!(err, Err(FetchError::BadCommit { .. })), "{err:?}");
}

#[test]
fn a_fetch_into_a_matching_directory_is_success() {
    let fixture = Fixture::new(false);
    assert_eq!(fixture.fetch("v1", None).unwrap(), SOURCE_HASH);
    assert_eq!(fixture.fetch("v1", None).unwrap(), SOURCE_HASH);
}

#[test]
fn two_fetches_at_once_both_pass() {
    let fixture = Fixture::new(false);
    let fetch_one = || fixture.fetch("v1", Some(SOURCE_HASH));
    let (left, right) = std::thread::scope(|scope| {
        let left = scope.spawn(fetch_one);
        let right = scope.spawn(fetch_one);
        (left.join().unwrap(), right.join().unwrap())
    });
    assert_eq!(left.unwrap(), SOURCE_HASH);
    assert_eq!(right.unwrap(), SOURCE_HASH);
}

#[test]
fn a_directory_whose_hash_differs_fails_naming_it() {
    let fixture = Fixture::new(false);
    let stray = format!("{SOURCE_HASH}/stray.toml");
    write_file(fixture.cache.path(), stray, "x = 1\n");
    let err = fixture.fetch("v1", None).unwrap_err();
    assert!(matches!(err, FetchError::Occupied { .. }), "{err:?}");
    assert!(err.to_string().contains(SOURCE_HASH), "{err}");
}

#[test]
fn a_missing_git_fails_with_a_hint_to_install_it() {
    let cache = tempdir();
    let commit = "1".repeat(40);
    let err = fetch_with(
        OsStr::new("mt-no-such-git"),
        "x",
        "v1",
        &commit,
        cache.path(),
    );
    let err = err.unwrap_err();
    assert_eq!(err.to_string(), "git not found");
    assert!(err.hint().is_some_and(|hint| hint.contains("install git")));
}

#[test]
fn a_failing_git_command_names_what_git_said() {
    let cache = tempdir();
    let missing = cache.path().join("no-such-repo.git").display().to_string();
    let commit = "1".repeat(40);
    let err = fetch(&missing, "v1", &commit, None, cache.path()).unwrap_err();
    assert!(err.to_string().starts_with("git fetch failed: "), "{err}");
}

#[test]
fn a_commit_id_is_hex_of_a_commit_ids_length() {
    let cache = tempdir();
    for commit in ["g".repeat(40), "1".repeat(39), "A".repeat(40)] {
        let err = fetch_with(
            OsStr::new("mt-no-such-git"),
            "x",
            "v1",
            &commit,
            cache.path(),
        );
        assert!(
            matches!(err, Err(FetchError::BadCommit { .. })),
            "{commit}: {err:?}"
        );
    }
}

#[test]
fn a_git_error_carries_the_first_line_git_wrote() {
    let err = git_error("fetch", b"\n  \nfatal: repository not found\nmore\n");
    assert_eq!(err.to_string(), "git fetch failed: repository not found");
    assert_eq!(
        git_error("fetch", b"").to_string(),
        "git fetch failed: no output"
    );
}

/// Hex digits to the bytes they spell.
fn unhex(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
        .collect()
}

/// A bare repository whose tag `v1` holds `../../../../evil`, written with
/// `--literally` because no porcelain command makes such a tree, and the
/// commit.
fn traversal_source() -> (tempfile::TempDir, String) {
    let (dir, work) = work_repo(&["init", "--quiet"]);
    write_file(&work, "evil", "pwned\n");
    let mut id = git_stdout(&work, &["hash-object", "-w", "evil"]);
    let mut entry = ("100644", "evil");
    for _ in 0..4 {
        let mut tree = format!("{} {}\0", entry.0, entry.1).into_bytes();
        tree.extend(unhex(&id));
        write_file(&work, "tree.bin", &tree);
        let args = ["hash-object", "-t", "tree", "--literally", "-w", "tree.bin"];
        id = git_stdout(&work, &args);
        entry = ("40000", "..");
    }
    let tester = [
        "-c",
        "user.name=t",
        "-c",
        "user.email=t@t",
        "commit-tree",
        &id,
        "-m",
        "x",
    ];
    let commit = git_stdout(&work, &tester);
    let commit = publish(dir.path(), &work, &commit);
    (dir, commit)
}

/// Every file name under `dir`, at any depth.
fn names_under(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        names.push(entry.file_name().to_string_lossy().into_owned());
        if entry.file_type().unwrap().is_dir() {
            names.extend(names_under(&entry.path()));
        }
    }
    names
}

#[test]
fn a_tree_entry_named_dot_dot_writes_nothing_outside_the_cache() {
    let (src, commit) = traversal_source();
    let root = tempdir();
    let cache = root.path().join("a/b/cache");
    let url = src.path().join("src.git").display().to_string();
    assert!(fetch(&url, "v1", &commit, None, &cache).is_err());
    let names = names_under(root.path());
    assert!(!names.iter().any(|name| name == "evil"), "{names:?}");
}

/// Set in the child [`fetch_ignores_the_callers_git_environment`] spawns.
const ENV_CHILD: &str = "MINDTAPE_TEST_FETCH_ENV_CHILD";

#[test]
fn fetch_ignores_the_callers_git_environment() {
    if let Ok(url) = std::env::var(ENV_CHILD) {
        let commit = std::env::var("MINDTAPE_TEST_FETCH_COMMIT").unwrap();
        let cache = std::env::var("MINDTAPE_TEST_FETCH_CACHE").unwrap();
        fetch(&url, "v1", &commit, Some(SOURCE_HASH), Path::new(&cache)).unwrap();
        return;
    }
    let fixture = Fixture::new(false);
    let elsewhere = tempdir();
    rerun_isolated_with("MINDTAPE_TEST_FETCH_ENV_SENTINEL", |child| {
        child
            .env(ENV_CHILD, fixture.url())
            .env("MINDTAPE_TEST_FETCH_COMMIT", &fixture.commit)
            .env("MINDTAPE_TEST_FETCH_CACHE", fixture.cache.path())
            .env("GIT_OBJECT_DIRECTORY", elsewhere.path())
            .env("GIT_ALLOW_PROTOCOL", "https");
    });
    assert!(fixture.cache.path().join(SOURCE_HASH).is_dir());
    assert_eq!(std::fs::read_dir(elsewhere.path()).unwrap().count(), 0);
}

#[test]
fn a_sha256_source_fetches_by_its_64_digit_commit() {
    let (dir, work) = work_repo(&["init", "--quiet", "--object-format=sha256"]);
    write_file(&work, "commits.toml", "a = 1\n");
    git_as_tester(&work, &["add", "."]);
    git_as_tester(&work, &["commit", "--quiet", "-m", "packs"]);
    let commit = publish(dir.path(), &work, "HEAD");
    assert_eq!(commit.len(), 64);
    let url = dir.path().join("src.git").display().to_string();
    let cache = tempdir();
    let sha = fetch(&url, "v1", &commit, None, cache.path()).unwrap();
    let file = cache.path().join(sha).join("commits.toml");
    assert_eq!(std::fs::read(file).unwrap(), b"a = 1\n");
}

#[test]
fn locate_joins_a_relative_path_to_the_root_and_passes_a_url() {
    let root = Path::new("/srv/project");
    let joined = root.join("../packs.git").display().to_string();
    assert_eq!(locate("../packs.git", root), joined);
    assert_eq!(
        locate("packs", root),
        root.join("packs").display().to_string()
    );
    for remote in [
        "https://example.com/acme/packs",
        "ssh://git@example.com/acme",
        "git@example.com:acme/packs.git",
        "example.com:packs",
    ] {
        assert_eq!(locate(remote, root), remote, "{remote}");
    }
    let absolute = std::env::temp_dir().join("packs.git").display().to_string();
    assert_eq!(locate(&absolute, root), absolute);
}

#[test]
fn locate_reads_a_slash_or_a_one_letter_prefix_before_the_colon_as_a_path() {
    let root = Path::new("/srv/project");
    for path in ["dir/a:b", "dir\\a:b", "C:packs"] {
        let joined = root.join(path).display().to_string();
        assert_eq!(locate(path, root), joined, "{path}");
    }
}
