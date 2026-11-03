//! Low-level git tree reads and path arithmetic shared by [`crate::commit::tree`]:
//! listing entries, reading blobs, resolving symlinks and mapping paths.

use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use crate::commit::CommitMsgError;

/// One path in a git listing, with its mode and blob id.
#[derive(Clone)]
pub(crate) struct GitEntry {
    pub(crate) path: String,
    pub(crate) oid: String,
    pub(crate) mode: String,
    pub(crate) kind: String,
}

/// Resolve a symlink `target` relative to repository-relative `link`, or
/// `None` when it is absolute or escapes the repository.
pub(crate) fn resolve_link(link: &str, target: &str) -> Option<String> {
    if Path::new(target).is_absolute() {
        return None;
    }
    let base = Path::new(link).parent().unwrap_or(Path::new(""));
    lexical_path(base.components().chain(Path::new(target).components()))
}

/// The object kind a git file mode names.
fn kind_of(mode: &str) -> &'static str {
    match mode {
        "040000" => "tree",
        "160000" => "commit",
        _ => "blob",
    }
}

/// Which git listing `parse_git_entries` is reading.
#[derive(Clone, Copy)]
pub(crate) enum Listing {
    /// `git ls-tree -r -z`: `<mode> <type> <oid>\t<path>`.
    Tree,
    /// `git ls-files -s -z`: `<mode> <oid> <stage>\t<path>`.
    Index,
}

/// Split a NUL-delimited git listing into entries of every kind: blobs,
/// trees and commits from `ls-tree`, and stage-zero index entries with their
/// mode-derived kind. [`crate::commit::tree::StateTree`] filters to blobs when
/// it lists task or package files.
pub(crate) fn parse_git_entries(out: &[u8], listing: Listing) -> Vec<GitEntry> {
    out.split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .filter_map(|entry| {
            let text = String::from_utf8_lossy(entry);
            let (meta, path) = text.split_once('\t')?;
            let mut fields = meta.split_whitespace();
            let mode = fields.next()?;
            let second = fields.next()?;
            let third = fields.next()?;
            let (kind, oid) = match listing {
                Listing::Tree => (second, third),
                Listing::Index if third == "0" => (kind_of(mode), second),
                Listing::Index => return None,
            };
            Some(GitEntry {
                path: path.to_owned(),
                oid: oid.to_owned(),
                mode: mode.to_owned(),
                kind: kind.to_owned(),
            })
        })
        .collect()
}

/// The project-relative path `path` names under `prefix`, or `None` when it
/// lies outside the project.
pub(crate) fn project_rel(prefix: &str, path: &str) -> Option<PathBuf> {
    let rel = path.strip_prefix(prefix)?.trim_start_matches('/');
    (!rel.is_empty()).then(|| PathBuf::from(rel))
}

/// The repository-relative path of project-relative `rel` under `prefix`.
pub(crate) fn project_repo_path(prefix: &str, rel: &Path) -> Option<String> {
    let rel = crate::slash_path(rel);
    (!rel.is_empty()).then(|| format!("{prefix}{rel}"))
}

/// The repository-relative path of `pkg` under `root_prefix`, or `None` when
/// it is absolute or escapes the repository. `.` and `..` are resolved
/// lexically, since a package may sit outside the project but inside the repo.
pub(crate) fn repo_path(root_prefix: &str, pkg: &Path) -> Option<String> {
    lexical_path(Path::new(root_prefix).components().chain(pkg.components()))
}

/// Join path components with `/`, resolving `.` and `..` lexically and
/// refusing an absolute or escaping path.
fn lexical_path<'a>(components: impl Iterator<Item = Component<'a>>) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    for component in components {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::Normal(name) => parts.push(name.to_str()?),
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(parts.join("/"))
}

/// The repository root above `root`, whose path within the repo is
/// `root_prefix`.
pub(crate) fn repo_root(root: &Path, root_prefix: &str) -> PathBuf {
    let mut repo = root.to_path_buf();
    for _ in Path::new(root_prefix).components() {
        repo.pop();
    }
    repo
}

/// The error for a git command that failed, carrying its stderr.
pub(crate) fn git_failed(command: &str, output: &std::process::Output) -> CommitMsgError {
    CommitMsgError::Git {
        command: command.to_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    }
}

/// Each object `specs` names, in order, through one `git cat-file --batch`:
/// its text, or `None` where the revision holds no such path.
pub(crate) fn read_blobs(
    root: &Path,
    specs: &[String],
) -> Result<Vec<Option<String>>, CommitMsgError> {
    Ok(read_objects(root, specs)?
        .into_iter()
        .map(|blob| blob.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
        .collect())
}

/// Each object `specs` names, in order, through one `git cat-file --batch`:
/// its bytes, or `None` where the revision holds no such path. A package may
/// carry images or fonts, so this keeps the bytes a task file would read as
/// text.
pub(crate) fn read_objects(
    root: &Path,
    specs: &[String],
) -> Result<Vec<Option<Vec<u8>>>, CommitMsgError> {
    if specs.is_empty() {
        return Ok(Vec::new());
    }
    let mut child = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let input: String = specs.iter().map(|spec| format!("{spec}\n")).collect();
    let mut stdin = child.stdin.take();
    // Written from its own thread: git answers as it reads, so a large batch
    // written inline would fill the output pipe before the input drains.
    let writer = std::thread::spawn(move || {
        stdin
            .as_mut()
            .map_or(Ok(()), |pipe| pipe.write_all(input.as_bytes()))
    });
    let mut out = Vec::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_to_end(&mut out)?;
    }
    let output = child.wait_with_output()?;
    writer.join().unwrap_or(Ok(()))?;
    if !output.status.success() {
        return Err(git_failed("cat-file", &output));
    }
    Ok(parse_batch_bytes(&out, specs.len()))
}

/// Split `git cat-file --batch` output into `count` objects: `<oid> <type>
/// <size>\n<content>\n` each, or `<spec> missing\n` for an absent one.
pub(crate) fn parse_batch_bytes(mut out: &[u8], count: usize) -> Vec<Option<Vec<u8>>> {
    let mut blobs = Vec::with_capacity(count);
    while blobs.len() < count {
        let Some(end) = out.iter().position(|byte| *byte == b'\n') else {
            break;
        };
        let header = String::from_utf8_lossy(out.get(..end).unwrap_or_default());
        out = out.get(end + 1..).unwrap_or_default();
        let size = header
            .rsplit(' ')
            .next()
            .and_then(|size| size.parse::<usize>().ok())
            .filter(|_| header.split(' ').count() == 3);
        let Some(size) = size else {
            blobs.push(None);
            continue;
        };
        let body = out.get(..size).unwrap_or_default();
        blobs.push(Some(body.to_vec()));
        out = out.get(size + 1..).unwrap_or_default();
    }
    blobs.resize(count, None);
    blobs
}
