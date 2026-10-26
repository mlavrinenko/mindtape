//! Running git for a fetch, apart from the caller's own git state: every
//! command drops the environment variables that point git at another
//! repository or config, and pins the settings that would change what it
//! fetches or how.
//!
//! Every command also runs in the cache root with `GIT_CEILING_DIRECTORIES`
//! set to its parent, so a command that takes no `--git-dir`, such as `ls-remote`,
//! never finds a repository around the directory `mt` runs in, nor reads
//! that repository's config, such as a `core.sshCommand` or a
//! `url.<base>.insteadOf` that would send a lookup and a fetch to two places.
//! The cache root is `mt`'s own, so no clone can put a repository there.

use std::ffi::OsStr;
use std::io::{BufRead, BufReader, ErrorKind, Read as _, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::pack_sources::FetchError;
use crate::config::pack_sources::listing::safe_join;

/// The settings every command runs with: line ends left alone, fetched
/// objects checked, and only the network transports and local paths a
/// source can name.
const PINNED: [&str; 7] = [
    "core.autocrlf=false",
    "core.eol=lf",
    "transfer.fsckObjects=true",
    "protocol.allow=never",
    "protocol.https.allow=always",
    "protocol.ssh.allow=always",
    "protocol.git.allow=always",
];

/// Local paths, which a test source and a source on disk use.
const FILE_PROTOCOL: &str = "protocol.file.allow=always";

/// The variable that stops git's search for a repository.
const CEILING: &str = "GIT_CEILING_DIRECTORIES";

/// A git program, the directory it runs in, the one git's search for a
/// repository stops below, and the variables it runs without.
pub(super) struct Git<'a> {
    program: &'a OsStr,
    workdir: PathBuf,
    ceiling: PathBuf,
    strip: Vec<String>,
}

impl<'a> Git<'a> {
    /// `program`, run in `workdir`, which it creates, with the variables
    /// `git rev-parse --local-env-vars` names and `GIT_ALLOW_PROTOCOL`
    /// stripped from each command it runs.
    pub(super) fn new(program: &'a OsStr, workdir: &Path) -> Result<Self, FetchError> {
        std::fs::create_dir_all(workdir)?;
        let workdir = std::path::absolute(workdir)?;
        // git never searches above a ceiling, but always searches the
        // directory it runs in, which is `mt`'s own.
        let ceiling = workdir.parent().unwrap_or(&workdir).to_path_buf();
        let output = Command::new(program)
            .current_dir(&workdir)
            .env(CEILING, &ceiling)
            .args(["rev-parse", "--local-env-vars"])
            .output()
            .map_err(spawn_error)?;
        if !output.status.success() {
            return Err(git_error("rev-parse", &output.stderr));
        }
        let mut strip: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_owned)
            .collect();
        strip.push("GIT_ALLOW_PROTOCOL".to_owned());
        Ok(Self {
            program,
            workdir,
            ceiling,
            strip,
        })
    }

    /// Run `args`, against `repo` as `--git-dir` when given, and return
    /// stdout.
    pub(super) fn run(
        &self,
        repo: Option<&Path>,
        args: &[&OsStr],
        step: &'static str,
    ) -> Result<Vec<u8>, FetchError> {
        let output = self
            .command(repo)
            .args(args)
            .output()
            .map_err(spawn_error)?;
        if output.status.success() {
            return Ok(output.stdout);
        }
        Err(git_error(step, &output.stderr))
    }

    /// Write each `(id, path)` blob of `repo` under `tree`, streamed from
    /// `git cat-file --batch`, which a second thread feeds so neither pipe
    /// fills while the other waits.
    pub(super) fn write_blobs(
        &self,
        repo: &Path,
        blobs: &[(String, String)],
        tree: &Path,
    ) -> Result<(), FetchError> {
        let mut child = self
            .command(Some(repo))
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(spawn_error)?;
        let ids: String = blobs.iter().map(|(id, _)| format!("{id}\n")).collect();
        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let written = std::thread::scope(|scope| {
            scope.spawn(move || stdin.map(|mut pipe| pipe.write_all(ids.as_bytes())));
            let mut reader = BufReader::new(stdout.ok_or_else(|| malformed("stdout"))?);
            for (id, path) in blobs {
                let target = safe_join(tree, path)?;
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut file = std::fs::File::create_new(&target)?;
                read_blob(&mut reader, id, &mut file)?;
            }
            Ok::<(), FetchError>(())
        });
        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(git_error("cat-file", &output.stderr));
        }
        written
    }

    fn command(&self, repo: Option<&Path>) -> Command {
        let mut command = Command::new(self.program);
        for name in &self.strip {
            command.env_remove(name);
        }
        command
            .current_dir(&self.workdir)
            .env(CEILING, &self.ceiling);
        for setting in PINNED.iter().chain([&FILE_PROTOCOL]) {
            command.args(["-c", setting]);
        }
        if let Some(repo) = repo {
            command.arg("--git-dir").arg(repo);
        }
        command
    }
}

/// Copy the next object of `git cat-file --batch` output to `out`. It must
/// be the blob `id`, its body exactly the size its header gives, and a
/// newline after it.
pub(super) fn read_blob(
    reader: &mut impl BufRead,
    id: &str,
    out: &mut impl Write,
) -> Result<(), FetchError> {
    let mut header = Vec::new();
    reader.read_until(b'\n', &mut header)?;
    let header = String::from_utf8_lossy(&header);
    let mut fields = header.strip_suffix('\n').unwrap_or("").split(' ');
    let size = match (fields.next(), fields.next(), fields.next(), fields.next()) {
        (Some(oid), Some("blob"), Some(size), None) if oid == id => size.parse::<u64>().ok(),
        _ => None,
    }
    .ok_or_else(|| malformed(id))?;
    let copied = std::io::copy(&mut reader.by_ref().take(size), out)?;
    let mut end = [0_u8];
    let ended = reader.read(&mut end)? == 1 && end == *b"\n";
    if copied != size || !ended {
        return Err(malformed(id));
    }
    Ok(())
}

/// A [`FetchError::Git`] for `cat-file` output that doesn't hold `what`.
fn malformed(what: &str) -> FetchError {
    FetchError::Git {
        step: "cat-file",
        message: format!("unexpected output for {}", crate::config::quote(what)),
    }
}

fn spawn_error(error: std::io::Error) -> FetchError {
    if error.kind() == ErrorKind::NotFound {
        FetchError::GitNotFound
    } else {
        FetchError::Io(error)
    }
}

/// A [`FetchError::Git`] carrying git's first stderr line.
pub(super) fn git_error(step: &'static str, stderr: &[u8]) -> FetchError {
    let stderr = String::from_utf8_lossy(stderr);
    let line = stderr
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("no output");
    FetchError::Git {
        step,
        message: line.trim().trim_start_matches("fatal: ").to_owned(),
    }
}

#[cfg(test)]
#[path = "git_tests.rs"]
mod tests;
