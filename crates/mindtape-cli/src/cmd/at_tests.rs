//! Tests for `mt at`: the `@name` sugar rewrite, the held set and its refusal,
//! the `at.toml` loader, and the subprocess-verified stdout/env paths.
#![allow(clippy::unwrap_used)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use mindtape::{AtConfig, Member, Target};

use super::desugar_with;
use crate::project_set::HeldSet;

fn argv(parts: &[&str]) -> Vec<OsString> {
    parts.iter().map(OsString::from).collect()
}

fn strs(argv: &[OsString]) -> Vec<String> {
    argv.iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect()
}

fn member(name: &str, fork: bool) -> Member {
    Member {
        name: name.to_owned(),
        path: PathBuf::from(format!("/p/{name}")),
        fork,
    }
}

/// Stub resolver: `home` is a set of two, any other token a project at
/// `/wt/<token>`. The `Result` wrapper mirrors the injected signature.
#[allow(clippy::unnecessary_wraps)]
fn stub(token: &str) -> crate::diag::Result<Target> {
    Ok(if token == "home" {
        Target::Set(vec![member("alpha", false), member("beta", true)])
    } else {
        Target::Project(PathBuf::from(format!("/wt/{token}")))
    })
}

fn desugar(parts: &[&str]) -> (Vec<String>, Option<HeldSet>) {
    let (out, held) = desugar_with(argv(parts), stub).unwrap();
    (strs(&out), held)
}

#[test]
fn a_project_rewrites_to_the_worktree_flag() {
    let (out, held) = desugar(&["mt", "@sn", "ls"]);
    assert_eq!(out, vec!["mt", "-w", "/wt/sn", "ls"]);
    assert_eq!(held, None);
}

#[test]
fn a_project_with_no_subcommand_still_rewrites() {
    assert_eq!(desugar(&["mt", "@sn"]).0, vec!["mt", "-w", "/wt/sn"]);
}

#[test]
fn a_set_is_held_and_leaves_no_worktree_flag() {
    let (out, held) = desugar(&["mt", "@home", "ls", "ready"]);
    assert_eq!(out, vec!["mt", "ls", "ready"]);
    assert_eq!(
        held,
        Some(HeldSet {
            name: "home".to_owned(),
            command: Some("ls".to_owned()),
            members: vec![
                ("alpha".to_owned(), PathBuf::from("/p/alpha")),
                ("beta".to_owned(), PathBuf::from("/p/beta")),
            ],
        })
    );
}

#[test]
fn a_held_set_names_the_command_past_global_flags() {
    let (_, held) = desugar(&["mt", "@home", "--json", "flip"]);
    assert_eq!(held.unwrap().command.as_deref(), Some("flip"));
}

#[test]
fn a_bare_set_holds_no_command() {
    let (out, held) = desugar(&["mt", "@home"]);
    assert_eq!(out, vec!["mt"]);
    assert_eq!(held.unwrap().command, None);
}

#[test]
fn the_refusal_names_the_command_the_count_and_a_project() {
    let (_, held) = desugar(&["mt", "@home", "flip"]);
    crate::diag::assert_plain(
        &held.unwrap().refusal(),
        "error: `flip` runs on one project, and @home holds 2 projects\n\
         hint: name one project, as in mt @alpha flip\n",
    );
}

#[test]
fn the_refusal_of_a_bare_set_says_this_command() {
    let (_, held) = desugar(&["mt", "@home"]);
    crate::diag::assert_plain(
        &held.unwrap().refusal(),
        "error: this command runs on one project, and @home holds 2 projects\n\
         hint: name one project, as in mt @alpha\n",
    );
}

#[test]
fn non_sugar_first_arg_is_untouched() {
    assert_eq!(desugar(&["mt", "ls", "@view"]).0, vec!["mt", "ls", "@view"]);
}

#[test]
fn at_only_after_subcommand_is_untouched() {
    // The view sigil in filter position must survive.
    let (out, held) = desugar(&["mt", "ls", "@broken", "--json"]);
    assert_eq!(out, vec!["mt", "ls", "@broken", "--json"]);
    assert_eq!(held, None);
}

#[test]
fn empty_argv_stays_empty() {
    assert!(desugar_with(Vec::new(), stub).unwrap().0.is_empty());
}

#[test]
fn resolution_failure_propagates() {
    let fail = |t: &str| -> crate::diag::Result<Target> {
        Err(crate::diag::Diag::failed("nope").value(t))
    };
    let err = desugar_with(argv(&["mt", "@bad", "ls"]), fail).unwrap_err();
    assert!(err.to_string().contains("nope: 'bad'"));
}

// --- unknown_name ---

fn config_with_sn() -> AtConfig {
    AtConfig::parse("[at.sn]\ninclude = [\"/wt/sn\"]\n").unwrap()
}

#[test]
fn unknown_name_shows_names_when_nonempty() {
    let err = super::unknown_name("bad", &config_with_sn(), None);
    crate::diag::assert_plain(
        &err,
        "error: unknown name: '@bad'\n\
         hint: add it to at.toml\n\
         note: known names: sn\n\
         note: no app config directory resolves\n",
    );
}

#[test]
fn unknown_name_omits_names_when_empty() {
    let err = super::unknown_name(
        "bad",
        &AtConfig::parse("").unwrap(),
        Some(Path::new("/c/at.toml")),
    );
    crate::diag::assert_plain(
        &err,
        "error: unknown name: '@bad'\n\
         hint: add it to at.toml\n\
         note: names are configured in: '/c/at.toml'\n",
    );
}

#[test]
fn unknown_name_suggests_a_near_miss() {
    let msg = crate::diag::plain(&super::unknown_name("sm", &config_with_sn(), None));
    assert!(msg.contains("hint: did you mean @sn?"), "{msg}");
}

#[test]
fn unknown_name_omits_suggestion_on_a_far_miss() {
    let msg = crate::diag::plain(&super::unknown_name("zzzzzzzz", &config_with_sn(), None));
    assert!(!msg.contains("did you mean"), "{msg}");
}

// ===== Subprocess tests (stdout / env isolation) =====
//
// Each test spawns itself as a child process with `$MINDTAPE_CONFIG_HOME`
// pointing at a temp directory and a sentinel env var set. The child
// branch runs the real function; the parent branch asserts the child's
// exit status and, for stdout tests, the captured output.

const CHILD: &str = "_MT_AT_CHILD";

/// [`mindtape::test_support::rerun_isolated_with`] under a temp
/// `$MINDTAPE_CONFIG_HOME` holding `files` (`rel`, `contents`).
fn run_child(files: &[(&str, &str)]) -> Option<(tempfile::TempDir, std::process::Output)> {
    if std::env::var(CHILD).is_ok() {
        return None;
    }
    let dir = mindtape::test_support::tempdir_with_files(files);
    let out = mindtape::test_support::rerun_isolated_with(CHILD, |child| {
        child.env(mindtape::CONFIG_HOME_ENV, dir.path());
    })?;
    Some((dir, out))
}

/// `config` as `mindtape/at.toml`.
fn at_file(config: &str) -> [(&str, &str); 1] {
    [("mindtape/at.toml", config)]
}

const SN_CFG: &str = "[at.sn]\ninclude = [\"/wt/sn\"]\n";

fn args(name: Option<&str>, json: bool) -> super::AtArgs {
    super::AtArgs {
        name: name.map(str::to_owned),
        json,
    }
}

#[test]
fn at_path_has_the_right_suffix() {
    let path = super::at_path();
    let p = path.as_ref().expect("at_path should be Some");
    assert!(p.ends_with("mindtape/at.toml"), "{p:?}");
}

#[test]
fn load_at_missing_file_returns_empty_and_path() {
    if run_child(&[]).is_some() {
        return;
    }
    let (config, path) = super::load_at().unwrap();
    assert!(config.names().is_empty(), "missing file → empty config");
    assert!(path.unwrap().ends_with("mindtape/at.toml"));
}

#[test]
fn load_at_valid_file_returns_entries() {
    if run_child(&at_file(SN_CFG)).is_some() {
        return;
    }
    let (config, path) = super::load_at().unwrap();
    assert_eq!(config.names(), vec!["sn"]);
    assert!(path.is_some());
}

#[test]
fn load_at_non_notfound_error_propagates() {
    if run_child(&[("mindtape", "i am a file not a dir")]).is_some() {
        return;
    }
    assert!(super::load_at().is_err(), "ENOTDIR should propagate");
}

/// A token that is not one path segment is refused, and the message names the
/// `@token` the user typed and says the resolution failed, not that the name is
/// unknown.
#[test]
fn resolve_token_refuses_a_token_that_is_not_one_path_segment() {
    if run_child(&at_file(SN_CFG)).is_some() {
        return;
    }
    let err = crate::diag::plain(&super::resolve_token("..").unwrap_err());
    assert!(err.contains("can't resolve"), "{err}");
    assert!(err.contains("'@..'"), "{err}");
}

#[test]
fn resolve_token_resolves_a_project() {
    if run_child(&at_file(SN_CFG)).is_some() {
        return;
    }
    let target = super::resolve_token("sn").unwrap();
    assert_eq!(target, Target::Project(PathBuf::from("/wt/sn")));
}

/// Run `mt at` as `args` under a config home holding `files`: the parent
/// hands what the child printed to `check`, the child runs the command.
fn prints(files: &[(&str, &str)], args: &super::AtArgs, check: impl Fn(&str)) {
    if let Some((_dir, out)) = run_child(files) {
        check(&String::from_utf8_lossy(&out.stdout));
        return;
    }
    super::run(args).unwrap();
}

/// [`prints`] `mt at home` over a [`fork_tree`] that `[at.home]` globs, with
/// `extra` keys on the entry.
fn home_prints(extra: &str, json: bool, check: impl Fn(&str)) {
    let tree = tempfile::TempDir::new().unwrap();
    fork_tree(tree.path());
    let glob = format!("{}/*", tree.path().display()).replace('\\', "/");
    let cfg = format!("[at.home]\ninclude = [\"{glob}\"]\n{extra}");
    prints(&at_file(&cfg), &args(Some("home"), json), check);
}

#[test]
fn run_lists_entries() {
    let cfg = format!("{SN_CFG}[at.home]\ninclude = [\"/wt/*\", \"@sn\"]\n");
    prints(&at_file(&cfg), &args(None, false), |stdout| {
        assert!(stdout.contains("@sn → /wt/sn\n"), "{stdout}");
        assert!(stdout.contains("@home → /wt/*, @sn\n"), "{stdout}");
    });
}

#[test]
fn run_with_json_lists_entries() {
    prints(&at_file(SN_CFG), &args(None, true), |stdout| {
        assert!(stdout.contains("\"name\": \"sn\""), "{stdout}");
        assert!(stdout.contains("\"forks\": \"false\""), "{stdout}");
    });
}

#[test]
fn run_with_malformed_config_errors() {
    if run_child(&at_file("not valid toml {{{")).is_some() {
        return;
    }
    assert!(super::run(&args(None, false)).is_err());
}

#[test]
fn run_without_config_names_the_config_dir() {
    prints(&[], &args(None, false), |stdout| {
        assert!(stdout.contains("nothing configured"), "{stdout}");
        assert!(stdout.contains("add [at.<name>] entries to"), "{stdout}");
    });
}

#[test]
fn run_without_resolvable_config_home_prints_fallback() {
    if let Some(out) = mindtape::test_support::rerun_isolated_with(CHILD, |child| {
        child.env(mindtape::CONFIG_HOME_ENV, "");
    }) {
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("could not resolve the app config directory"),
            "{stdout}"
        );
        return;
    }
    super::run(&args(None, false)).unwrap();
}

/// Two projects under `root`, `fork` a linked git worktree of `main`.
fn fork_tree(root: &Path) {
    for dir in [
        "main/.mindtape",
        "fork/.mindtape",
        "main/.git/worktrees/fork",
    ] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    std::fs::write(
        root.join("fork/.git"),
        "gitdir: ../main/.git/worktrees/fork\n",
    )
    .unwrap();
    std::fs::write(root.join("main/.git/worktrees/fork/commondir"), "../..\n").unwrap();
}

#[test]
fn run_with_a_name_lists_members_and_marks_forks() {
    home_prints("forks = true\n", false, |stdout| {
        assert!(stdout.contains("@home holds 2 projects\n"), "{stdout}");
        assert!(stdout.contains("fork  (fork)\n"), "{stdout}");
        assert!(!stdout.contains("main  (fork)"), "{stdout}");
    });
}

#[test]
fn run_with_a_name_and_json_prints_members() {
    home_prints("", true, |stdout| {
        assert!(stdout.contains("\"kind\": \"set\""), "{stdout}");
        assert!(stdout.contains("\"name\": \"main\""), "{stdout}");
        assert!(stdout.contains("\"fork\": false"), "{stdout}");
        assert!(!stdout.contains("\"name\": \"fork\""), "{stdout}");
    });
}

#[test]
fn run_with_a_project_name_prints_its_path() {
    prints(&at_file(SN_CFG), &args(Some("sn"), false), |stdout| {
        assert!(stdout.contains("@sn → /wt/sn\n"), "{stdout}");
    });
}

#[test]
fn run_with_a_project_name_and_json_prints_its_path() {
    prints(&at_file(SN_CFG), &args(Some("sn"), true), |stdout| {
        assert!(stdout.contains("\"kind\": \"project\""), "{stdout}");
        assert!(stdout.contains("\"path\": \"/wt/sn\""), "{stdout}");
    });
}

#[test]
fn run_with_an_unknown_name_errors() {
    if run_child(&at_file(SN_CFG)).is_some() {
        return;
    }
    assert!(super::run(&args(Some("nope"), false)).is_err());
}
