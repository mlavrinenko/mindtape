//! The one door for spawning a script or a shell from an e2e test.
//!
//! The host's `BASH_ENV` hook (`/etc/bash_env`) runs when bash starts, before
//! any script line: it re-derives `PATH` from the `.envrc` above the working
//! directory, and a throwaway directory has none, so the dev shell's tools
//! vanish. Setting `BASH_ENV=` inside the script is too late. [`shell`] strips
//! it from the environment instead, and puts the developer's git config out of
//! reach. The guard test below refuses a direct `Command::new("bash")`.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::CLEARED_VARS;

/// The checkout this test binary was built from.
pub fn checkout() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `program` run in `dir` with no `BASH_ENV`, no developer HOME or system git
/// config, and none of the [`CLEARED_VARS`]. A test adds its own `env` after.
pub fn shell(program: &str, dir: &Path) -> Command {
    let home = dir.join("no-home");
    let mut command = Command::new(program);
    command
        .current_dir(dir)
        .env_remove("BASH_ENV")
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", &home)
        .env("GIT_CONFIG_NOSYSTEM", "1");
    for key in CLEARED_VARS {
        command.env_remove(key);
    }
    command
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn strips_bash_env_and_the_developer_home() {
        let command = shell("bash", Path::new("/tmp/x"));
        let envs: Vec<_> = command.get_envs().collect();
        assert!(envs.contains(&(OsStr::new("BASH_ENV"), None)));
        assert!(envs.contains(&(OsStr::new("DIRENV_DIFF"), None)));
        assert!(envs.contains(&(OsStr::new("HOME"), Some(OsStr::new("/tmp/x/no-home")))));
    }

    /// No e2e module may spawn `bash` or `sh` itself: the host hook would
    /// rewrite its `PATH`. Route it through [`shell`].
    #[test]
    fn no_test_spawns_a_shell_directly() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/it");
        let mut offenders = Vec::new();
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension() != Some(OsStr::new("rs")) {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for (n, line) in text.lines().enumerate() {
                let direct = [r#"Command::new("bash")"#, r#"Command::new("sh")"#];
                if direct.iter().any(|d| line.contains(d)) {
                    offenders.push(format!("{}:{}", path.display(), n + 1));
                }
            }
        }
        assert!(offenders.is_empty(), "use support::shell: {offenders:?}");
    }
}
