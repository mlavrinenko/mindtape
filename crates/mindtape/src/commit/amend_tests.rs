#![allow(clippy::unwrap_used)]

use super::*;

fn argv(line: &str) -> Vec<String> {
    line.split_whitespace().map(ToOwned::to_owned).collect()
}

#[test]
fn the_nearest_git_commit_decides() {
    let chain = [
        argv("mt check"),
        argv("sh .git/hooks/commit-msg x"),
        argv("git commit --amend --no-edit"),
    ];
    assert!(is_amend(&chain));
}

#[test]
fn a_plain_commit_is_not_an_amend() {
    let chain = [
        argv("mt check"),
        argv("/usr/bin/git -C d commit -m x"),
        argv("git commit --amend"),
    ];
    assert!(!is_amend(&chain), "the nearer commit has no --amend");
}

#[test]
fn a_git_process_that_is_not_a_commit_is_skipped() {
    let chain = [argv("git log --amend"), argv("bash")];
    assert!(!is_amend(&chain));
    assert!(!is_amend(&[]));
}

#[test]
fn a_program_that_is_not_git_is_not_a_commit() {
    assert!(!is_amend(&[argv("mygit commit --amend")]));
    assert!(is_amend(&[argv(r"C:\Git\git.exe commit --amend")]));
}

#[test]
fn a_flag_only_matches_exactly() {
    assert!(!is_amend(&[argv("git commit --amend-x")]));
}

#[test]
fn this_process_has_a_readable_parent() {
    assert!(process(std::process::id()).is_some() || !Path::new("/proc/self").exists());
}

#[cfg(unix)]
#[test]
fn the_parent_pid_is_read_from_proc_and_from_ps() {
    let own = std::process::id();
    let parent = std::os::unix::process::parent_id();
    if Path::new("/proc/self/stat").exists() {
        let (read, argv) = from_proc(own).unwrap();
        assert_eq!(read, parent);
        assert!(!argv.is_empty());
    }
    // `ps` may be missing from a sandbox; where it is installed it must agree.
    if Command::new("ps").arg("-p").arg("1").output().is_ok() {
        let (read, argv) = from_ps(own).unwrap();
        assert_eq!(read, parent);
        assert!(!argv.is_empty());
    }
}
