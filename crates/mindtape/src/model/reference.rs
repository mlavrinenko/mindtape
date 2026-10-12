//! Task references: the path, stem or stem hash typed to name one task, and the
//! one resolver every reader of them calls.
//!
//! A reference reads the same wherever it is typed: a path, else an exact stem,
//! else a [stem hash](crate::model::handle). Only its [`Base`] differs. A
//! `<task>` on the command line reads a path from the working directory, and a
//! `links:` target reads it from the linking file, as `mt check` reads the
//! written edge. A stem several tasks share names the one in the base's folder,
//! else none of them.

use std::path::{Path, PathBuf};

use crate::model::handle::{Resolution, StemHashes};
use crate::model::{Task, relative_target, resolve_link_path};

/// Where a reference is read from.
#[derive(Debug, Clone, Copy)]
pub enum Base<'a> {
    /// A `<task>` typed on the command line: a path is an OS path from this
    /// absolute directory.
    Cwd(&'a Path),
    /// A `links:` target written in this root-relative task file: a path reads
    /// from the file's folder, without touching the disk.
    Linker(&'a Path),
}

impl<'a> Base<'a> {
    /// The base's folder, root-relative, or `None` when it lies outside `root`.
    fn folder<'p>(self, root: &'p Path) -> Option<&'p Path>
    where
        'a: 'p,
    {
        match self {
            Self::Cwd(dir) => dir.strip_prefix(root).ok(),
            Self::Linker(file) => Some(file.parent().unwrap_or(Path::new(""))),
        }
    }

    /// The path from the base to the root-relative `file`: what to type there,
    /// and for a [`Base::Linker`] what its edge writes.
    #[must_use]
    pub fn path_to(self, root: &Path, file: &Path) -> PathBuf {
        match self.folder(root) {
            Some(folder) => relative_target(folder, file),
            None => root.join(file),
        }
    }
}

/// How a reference named its task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    /// It is spelled as a path.
    Path,
    /// It is the task's stem.
    Stem,
    /// It is a stem hash, tried because no stem matched.
    Hash,
}

/// Why a task reference named no task.
#[derive(Debug, thiserror::Error)]
pub enum RefError {
    /// A reference spelled as a path could not be resolved.
    #[error("resolve {}", .path.display())]
    Path {
        /// The path, joined onto the working directory when relative.
        path: PathBuf,
        /// Why it did not resolve.
        #[source]
        source: std::io::Error,
    },
    /// A path that no loaded task owns.
    #[error("no task at {}", .0.display())]
    NoTaskAt(PathBuf),
    /// A stem several tasks share, none of them in the base's folder.
    #[error("stem {reference:?} names {} tasks", .paths.len())]
    SharedStem {
        /// The typed reference.
        reference: String,
        /// The path from the base to each task, sorted.
        paths: Vec<PathBuf>,
    },
    /// A stem hash several tasks share.
    #[error("stem hash {reference:?} names {} tasks", .candidates.len())]
    Ambiguous {
        /// The typed reference.
        reference: String,
        /// The stem hash and the stem of each task it names, sorted by stem.
        candidates: Vec<(String, String)>,
    },
    /// The reference is empty, so it names nothing to look for.
    #[error("empty task reference")]
    Empty,
    /// Neither a stem nor a stem hash matches.
    #[error("no task with id {reference:?}")]
    NoMatch {
        /// The typed reference.
        reference: String,
        /// The loaded stem nearest to it, when one is close.
        nearest: Option<String>,
    },
}

/// The task a reference names, and its file.
#[derive(Debug, Clone)]
pub struct Resolved<'a> {
    /// The task.
    pub task: &'a Task,
    /// Its file: canonical when the reference was a path from the working
    /// directory, else under the root.
    pub path: PathBuf,
    /// How the reference named it.
    pub via: Via,
}

/// Whether `reference` is spelled as a path: absolute, holding a separator, or
/// ending `.typ`. Anything else is a stem or a stem hash.
#[must_use]
pub fn is_path_ref(reference: &str) -> bool {
    Path::new(reference).is_absolute()
        || reference.contains(['/', '\\'])
        || reference.ends_with(".typ")
}

/// Resolves `reference`, read from `base`, to one of `tasks`: a path, else an
/// exact stem, else a stem hash from `hashes`. Task files are `root`-relative.
///
/// # Errors
/// Returns a [`RefError`] when the reference is empty, a path does not resolve
/// or owns no task, a stem or a hash names several tasks, or nothing matches.
pub fn resolve_ref<'a>(
    tasks: &'a [Task],
    root: &Path,
    base: Base<'_>,
    hashes: &StemHashes,
    reference: &str,
) -> Result<Resolved<'a>, RefError> {
    let lookup = Lookup { tasks, root, base };
    if reference.is_empty() {
        return Err(RefError::Empty);
    }
    if is_path_ref(reference) {
        return match base {
            Base::Cwd(dir) => lookup.os_path(dir, reference),
            Base::Linker(file) => lookup.written_path(file, reference),
        };
    }
    match lookup.stem(reference, reference)? {
        Some(task) => Ok(lookup.resolved(task, Via::Stem)),
        None => lookup
            .hash(hashes, reference)
            .map(|task| lookup.resolved(task, Via::Hash)),
    }
}

/// The tasks a reference is resolved against, and where it is read from.
struct Lookup<'a, 'b> {
    tasks: &'a [Task],
    root: &'b Path,
    base: Base<'b>,
}

impl<'a> Lookup<'a, '_> {
    fn resolved(&self, task: &'a Task, via: Via) -> Resolved<'a> {
        Resolved {
            task,
            path: self.root.join(&task.file),
            via,
        }
    }

    /// An OS path from `dir`, matched on canonical files.
    fn os_path(&self, dir: &Path, reference: &str) -> Result<Resolved<'a>, RefError> {
        let joined = dir.join(reference);
        let abs = joined.canonicalize().map_err(|source| RefError::Path {
            path: joined,
            source,
        })?;
        let task = self
            .tasks
            .iter()
            .find(|task| self.root.join(&task.file).canonicalize().ok().as_deref() == Some(&abs))
            .ok_or_else(|| RefError::NoTaskAt(abs.clone()))?;
        Ok(Resolved {
            task,
            path: abs,
            via: Via::Path,
        })
    }

    /// A path written in the root-relative `linker`, read lexically.
    fn written_path(&self, linker: &Path, reference: &str) -> Result<Resolved<'a>, RefError> {
        let file = resolve_link_path(linker, reference)
            .ok_or_else(|| RefError::NoTaskAt(PathBuf::from(reference)))?;
        let task = self
            .tasks
            .iter()
            .find(|task| task.file == file)
            .ok_or(RefError::NoTaskAt(file))?;
        Ok(self.resolved(task, Via::Path))
    }

    /// The task whose stem is `stem`, `None` when no task has it. A stem
    /// several tasks share names the one in the base's folder, else is refused
    /// as `reference`.
    fn stem(&self, stem: &str, reference: &str) -> Result<Option<&'a Task>, RefError> {
        let mut found: Vec<&'a Task> = self.tasks.iter().filter(|task| task.id == stem).collect();
        if found.len() < 2 {
            return Ok(found.pop());
        }
        let folder = self.base.folder(self.root);
        if let Some(beside) = found.iter().find(|task| task.file.parent() == folder) {
            return Ok(Some(beside));
        }
        let mut paths: Vec<PathBuf> = found
            .iter()
            .map(|task| self.base.path_to(self.root, &task.file))
            .collect();
        paths.sort();
        Err(RefError::SharedStem {
            reference: reference.to_owned(),
            paths,
        })
    }

    /// The task whose stem hash `reference` is.
    fn hash(&self, hashes: &StemHashes, reference: &str) -> Result<&'a Task, RefError> {
        match hashes.resolve(reference) {
            Resolution::Found(stem) => self
                .stem(&stem, reference)?
                .ok_or_else(|| self.no_match(reference)),
            Resolution::Ambiguous(stems) => Err(RefError::Ambiguous {
                reference: reference.to_owned(),
                candidates: stems
                    .into_iter()
                    .map(|stem| (hashes.hash_of(&stem).unwrap_or_default().to_owned(), stem))
                    .collect(),
            }),
            Resolution::NoMatch => Err(self.no_match(reference)),
        }
    }

    fn no_match(&self, reference: &str) -> RefError {
        let stems: Vec<&str> = self.tasks.iter().map(|task| task.id.as_str()).collect();
        RefError::NoMatch {
            reference: reference.to_owned(),
            nearest: crate::suggest::nearest(reference, &stems).map(ToOwned::to_owned),
        }
    }
}

#[cfg(test)]
#[path = "reference_tests.rs"]
mod tests;
