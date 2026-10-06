//! `outdatty.lock` merges key by key through the `outdatty-lock` merge driver
//! (`.just/scripts/outdatty-lock-merge.sh`), which `just install-hooks`
//! registers and `.gitattributes` assigns. Each test builds a throwaway repo
//! that carries the driver, the attributes and two outdatty groups, registers
//! the driver with the real recipe, and merges two branches that recorded the
//! lock.
//!
//! Unix only: the driver needs `bash`, `yq`, `jq` and `b3sum`, which the dev
//! shell provides and the Windows job, which has no dev shell, does not.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The checkout this test binary was built from.
fn checkout() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const DRIVER: &str = ".just/scripts/outdatty-lock-merge.sh";

const MANIFEST: &str = "require_tracked: [\"!**\"]
groups:
  - name: alpha
    source: [a.txt]
    dependents: [a-doc.txt]
  - name: gamma
    source: [g.txt]
    dependents: [g-doc.txt]
";

/// A five-line file, so an edit to its first line and one to its last merge
/// as text without a conflict.
const FIVE_LINES: &str = "one\ntwo\nthree\nfour\nfive\n";

struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    /// A repo on `main` holding the driver, `.gitattributes`, [`MANIFEST`],
    /// its four files and a lock that records them all, committed as `base`,
    /// with the driver registered by `just install-hooks`.
    fn new() -> Self {
        let repo = Self {
            dir: tempfile::tempdir().unwrap(),
        };
        repo.git(&["init", "-q", "-b", "main"]);
        repo.git(&["config", "user.email", "t@example.com"]);
        repo.git(&["config", "user.name", "t"]);
        repo.git(&["config", "commit.gpgsign", "false"]);
        for path in [DRIVER, ".gitattributes"] {
            repo.write(
                path,
                &std::fs::read_to_string(checkout().join(path)).unwrap(),
            );
        }
        repo.write("outdatty.yaml", MANIFEST);
        for path in ["a.txt", "a-doc.txt", "g.txt", "g-doc.txt"] {
            repo.write(path, FIVE_LINES);
        }
        let justfile = checkout().join("Justfile");
        let justfile = justfile.to_str().unwrap();
        let root = repo.dir.path().to_str().unwrap();
        repo.ok(
            "just",
            &[
                "--justfile",
                justfile,
                "--working-directory",
                root,
                "install-hooks",
            ],
        );
        repo.ok("outdatty", &["update", "--format", "quiet"]);
        repo.commit("base");
        repo
    }

    /// Run `program` in the repo with the developer's git config out of
    /// reach. `BASH_ENV` goes too: git runs the driver through `sh -c`, and a
    /// host hook there may re-derive `PATH` from the `.envrc` this throwaway
    /// repo lacks, dropping the dev-shell tools the driver needs.
    fn command(&self, program: &str, args: &[&str]) -> Output {
        let home = self.dir.path().join("no-home");
        Command::new(program)
            .env_remove("BASH_ENV")
            .args(args)
            .current_dir(self.dir.path())
            .env("HOME", &home)
            .env("XDG_CONFIG_HOME", &home)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap()
    }

    /// Run `program`, assert it succeeded, and return its stdout.
    fn ok(&self, program: &str, args: &[&str]) -> String {
        let out = self.command(program, args);
        assert!(
            out.status.success(),
            "{program} {args:?} failed:\n{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    fn git(&self, args: &[&str]) -> String {
        self.ok("git", args)
    }

    fn write(&self, path: &str, content: &str) {
        let path = self.dir.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn read(&self, path: &str) -> String {
        std::fs::read_to_string(self.dir.path().join(path)).unwrap()
    }

    fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", message]);
    }

    /// Record `group` in the lock, or only `dependent` of it.
    fn record(&self, group: &str, dependent: Option<&str>) {
        let mut args = vec!["update", "--format", "quiet", "--group", group];
        if let Some(path) = dependent {
            args.extend(["--dependent", path]);
        }
        self.ok("outdatty", &args);
    }

    /// Whether the three lock blobs conflict under git's own text merge,
    /// which is what a merge without the driver would do.
    fn text_merge_conflicts(&self, base: &str, ours: &str, theirs: &str) -> bool {
        for (name, rev) in [("O", base), ("A", ours), ("B", theirs)] {
            let blob = self.git(&["show", &format!("{rev}:outdatty.lock")]);
            self.write(&format!("merge-{name}"), &blob);
        }
        let out = self.command(
            "git",
            &["merge-file", "-p", "merge-A", "merge-O", "merge-B"],
        );
        for name in ["O", "A", "B"] {
            std::fs::remove_file(self.dir.path().join(format!("merge-{name}"))).unwrap();
        }
        !out.status.success()
    }

    /// The lock in the working tree is what outdatty itself writes for the
    /// merged tree: `outdatty check` passes and a full re-record changes no
    /// byte of it.
    fn assert_lock_is_current(&self) {
        let before = self.read("outdatty.lock");
        assert!(!before.contains("<<<<<<<"), "{before}");
        self.ok("outdatty", &["check", "--format", "quiet"]);
        self.ok("outdatty", &["update", "--format", "quiet"]);
        assert_eq!(self.read("outdatty.lock"), before);
    }
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn the_driver_merges_three_lock_versions_that_conflict_as_text() {
    let repo = Repo::new();
    let hash = |pair: &str| pair.repeat(32);
    let base = format!(
        "version: 1\nalgorithm: blake3\ngroups:\n  alpha:\n    source:\n      a.txt: {}\n    dependents:\n      a-doc.txt: {}\n  gamma:\n    source:\n      g.txt: {}\n    dependents: {{}}\n",
        hash("1f"),
        hash("2e"),
        hash("3d")
    );
    let beta = format!(
        "  beta:\n    source:\n      b.txt: {}\n    dependents: {{}}\n  gamma:\n",
        hash("b0")
    );
    let ours = base.replace(&hash("2e"), &hash("a0"));
    let theirs = base
        .replace("  gamma:\n", &beta)
        .replace(&hash("3d"), &hash("c0"));
    repo.write("O", &base);
    repo.write("A", &ours);
    repo.write("B", &theirs);
    let text = repo.command("git", &["merge-file", "-p", "A", "O", "B"]);
    assert!(!text.status.success(), "the fixture must conflict as text");

    let out = repo.command(
        "bash",
        &[
            DRIVER,
            "O",
            "A",
            "B",
            "outdatty.lock",
            "base",
            "ours",
            "theirs",
        ],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let expected = ours
        .replace("  gamma:\n", &beta)
        .replace(&hash("3d"), &hash("c0"));
    assert_eq!(repo.read("A"), expected);
}

#[test]
fn a_lock_merged_with_itself_keeps_every_byte() {
    let repo = Repo::new();
    let lock = repo.read("outdatty.lock");
    for name in ["O", "A", "B"] {
        repo.write(name, &lock);
    }
    let out = repo.command("bash", &[DRIVER, "O", "A", "B", "outdatty.lock"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(repo.read("A"), lock);
}

#[test]
fn a_rebase_over_branches_that_recorded_different_groups_does_not_conflict() {
    let repo = Repo::new();
    repo.git(&["checkout", "-q", "-b", "feat"]);
    repo.write(
        "outdatty.yaml",
        &MANIFEST.replace(
            "  - name: gamma",
            "  - name: beta\n    source: [b.txt]\n    dependents: [b-doc.txt]\n  - name: gamma",
        ),
    );
    repo.write("b.txt", FIVE_LINES);
    repo.write("b-doc.txt", FIVE_LINES);
    repo.record("beta", None);
    repo.commit("feat: add beta");

    repo.git(&["checkout", "-q", "main"]);
    repo.write("a.txt", "changed\n");
    repo.write("a-doc.txt", "changed\n");
    repo.record("alpha", None);
    repo.commit("feat: change alpha");
    assert!(repo.text_merge_conflicts("main~1", "main", "feat"));

    repo.git(&["checkout", "-q", "feat"]);
    let out = repo.command("git", &["rebase", "main"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    repo.assert_lock_is_current();
}

/// The base lock holds a stale hash for `a-doc.txt`, so a re-record of
/// `alpha` on one side and a recorded edit of `a-doc.txt` on the other both
/// change that key. The merged tree holds the edited file, so that side wins.
fn both_sides_record_a_doc_against_a_stale_base(repo: &Repo) {
    repo.write("a-doc.txt", "stale\n");
    repo.commit("docs: edit a-doc without recording it");
    repo.git(&["checkout", "-q", "-b", "feat"]);
    repo.write("a-doc.txt", "reviewed\n");
    repo.record("alpha", Some("a-doc.txt"));
    repo.commit("docs: review a-doc");
    repo.git(&["checkout", "-q", "main"]);
    repo.write("a.txt", "changed\n");
    repo.record("alpha", None);
    repo.commit("feat: change alpha");
}

#[test]
fn a_rebase_takes_the_side_whose_hash_matches_the_merged_file() {
    let repo = Repo::new();
    both_sides_record_a_doc_against_a_stale_base(&repo);
    repo.git(&["checkout", "-q", "feat"]);
    let out = repo.command("git", &["rebase", "main"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    assert_eq!(repo.read("a-doc.txt"), "reviewed\n");
    repo.assert_lock_is_current();
}

#[test]
fn a_merge_takes_the_side_whose_hash_matches_the_merged_file() {
    let repo = Repo::new();
    both_sides_record_a_doc_against_a_stale_base(&repo);
    let out = repo.command("git", &["merge", "-q", "--no-edit", "feat"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    assert_eq!(repo.read("a-doc.txt"), "reviewed\n");
    repo.assert_lock_is_current();
}

#[test]
fn a_key_both_sides_recorded_against_neither_merged_file_conflicts() {
    let repo = Repo::new();
    repo.git(&["checkout", "-q", "-b", "feat"]);
    repo.write("a-doc.txt", &FIVE_LINES.replace("five", "FIVE"));
    repo.record("alpha", Some("a-doc.txt"));
    repo.commit("docs: edit the last line");
    repo.git(&["checkout", "-q", "main"]);
    repo.write("a-doc.txt", &FIVE_LINES.replace("one", "ONE"));
    repo.record("alpha", Some("a-doc.txt"));
    repo.commit("docs: edit the first line");

    repo.git(&["checkout", "-q", "feat"]);
    let out = repo.command("git", &["rebase", "main"]);
    assert!(!out.status.success(), "{}", stdout(&out));
    let text = stderr(&out);
    assert!(
        text.contains("group alpha") && text.contains("a-doc.txt"),
        "the conflict names the group and the path: {text}"
    );
    let unmerged = repo.git(&["diff", "--name-only", "--diff-filter=U"]);
    assert_eq!(
        unmerged, "outdatty.lock\n",
        "a-doc.txt itself merged as text"
    );
    assert_eq!(repo.read("a-doc.txt"), "ONE\ntwo\nthree\nfour\nFIVE\n");
    assert!(repo.read("outdatty.lock").contains("<<<<<<<"));
}
