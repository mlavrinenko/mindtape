//! `@x` to a project or a set: `[at.x]` first, else the directory whose path
//! ends in `x` that an `include` matches, so `@blog` and `@oss/blog` both find
//! `~/oss/blog`. A set expands its `@name` includes, drops what `exclude`
//! matches and filters a glob's forks, then names each member by the shortest
//! path suffix that `@` takes back.

use std::path::{Component, Path, PathBuf};

use super::scan::{expand_glob, expand_tilde, is_fork, is_glob};
use super::{AtConfig, AtError, AtFs, Forks, Member, Target, validate_token};

impl AtConfig {
    /// Resolve `token`, the text after a leading `@`.
    ///
    /// An entry whose `include` is one literal path is a project, and so is a
    /// directory found by its name or a longer path suffix, such as
    /// `oss/blog`. Any other entry is a set. `home` expands a leading `~`.
    /// Returns `Ok(None)` when nothing answers to `token`.
    ///
    /// # Errors
    /// [`AtError::InvalidName`] when `token` is not path segments joined by
    /// `/`,
    /// [`AtError::Clash`] when two directories answer to it,
    /// [`AtError::Cycle`] on an `@name` cycle, [`AtError::Unknown`] on an
    /// `@name` include that names nothing, or [`AtError::Glob`] on a bad glob.
    pub fn resolve(
        &self,
        token: &str,
        home: Option<&Path>,
        fs: &dyn AtFs,
    ) -> Result<Option<Target>, AtError> {
        validate_token(token)?;
        let resolver = Resolver {
            config: self,
            home,
            fs,
        };
        let Some(entry) = self.entries.get(token) else {
            return Ok(resolver.find_dir(token)?.map(Target::Project));
        };
        if let [only] = entry.include.as_slice()
            && is_literal(only)
        {
            return Ok(Some(Target::Project(expand_tilde(only, home))));
        }
        let paths = resolver.expand(token, &mut Vec::new())?;
        Ok(Some(Target::Set(name_members(paths, fs))))
    }
}

/// One resolution's inputs.
struct Resolver<'a> {
    config: &'a AtConfig,
    home: Option<&'a Path>,
    fs: &'a dyn AtFs,
}

impl Resolver<'_> {
    /// The member paths of the entry `name`, in include order. `stack` holds
    /// the entries being expanded, to refuse a cycle.
    fn expand(&self, name: &str, stack: &mut Vec<String>) -> Result<Vec<PathBuf>, AtError> {
        if let Some(start) = stack.iter().position(|seen| seen == name) {
            let mut chain = stack.split_off(start);
            chain.push(name.to_owned());
            return Err(AtError::Cycle { chain });
        }
        let Some(entry) = self.config.entries.get(name) else {
            return Err(AtError::Unknown {
                name: name.to_owned(),
            });
        };
        stack.push(name.to_owned());
        let mut paths = Vec::new();
        for pattern in &entry.include {
            for path in self.pattern(pattern, Some(entry.forks), stack)? {
                if !paths.contains(&path) {
                    paths.push(path);
                }
            }
        }
        let mut excluded = Vec::new();
        for pattern in &entry.exclude {
            excluded.extend(self.pattern(pattern, None, stack)?);
        }
        paths.retain(|path| !excluded.contains(path));
        stack.pop();
        Ok(paths)
    }

    /// The paths one `include` or `exclude` item names. `forks` filters a
    /// glob's matches; `None` keeps them all.
    fn pattern(
        &self,
        pattern: &str,
        forks: Option<Forks>,
        stack: &mut Vec<String>,
    ) -> Result<Vec<PathBuf>, AtError> {
        if let Some(name) = pattern.strip_prefix('@') {
            if self.config.entries.contains_key(name) {
                return self.expand(name, stack);
            }
            return match self.find_dir(name)? {
                Some(path) => Ok(vec![path]),
                None => Err(AtError::Unknown {
                    name: name.to_owned(),
                }),
            };
        }
        let path = expand_tilde(pattern, self.home);
        if !is_glob(pattern) {
            return Ok(vec![path]);
        }
        let mut found = expand_glob(&path, self.fs)?;
        match forks {
            Some(Forks::Skip) => found.retain(|dir| !is_fork(dir, self.fs)),
            Some(Forks::Only) => found.retain(|dir| is_fork(dir, self.fs)),
            Some(Forks::Keep) | None => {}
        }
        Ok(found)
    }

    /// The one directory whose path ends in `name`'s segments that any
    /// entry's literal or glob `include` matches, whatever its `exclude` and
    /// `forks` say.
    fn find_dir(&self, name: &str) -> Result<Option<PathBuf>, AtError> {
        let wanted: Vec<String> = name.split('/').map(str::to_owned).collect();
        let mut hits: Vec<PathBuf> = Vec::new();
        for entry in self.config.entries.values() {
            for pattern in entry.include.iter().filter(|p| !p.starts_with('@')) {
                let path = expand_tilde(pattern, self.home);
                let candidates = if is_glob(pattern) {
                    expand_glob(&path, self.fs)?
                } else {
                    vec![path]
                };
                for candidate in candidates {
                    if segments_of(&candidate).ends_with(&wanted) && !hits.contains(&candidate) {
                        hits.push(candidate);
                    }
                }
            }
        }
        match hits.len() {
            0 => Ok(None),
            1 => Ok(hits.pop()),
            _ => Err(AtError::Clash {
                name: name.to_owned(),
                candidates: name_members(hits, self.fs),
            }),
        }
    }
}

/// Whether an `include` item is one literal path, not a glob or an `@name`.
fn is_literal(pattern: &str) -> bool {
    !pattern.starts_with('@') && !is_glob(pattern)
}

/// Name each member by its directory, or by its shortest path suffix that no
/// other member shares.
fn name_members(paths: Vec<PathBuf>, fs: &dyn AtFs) -> Vec<Member> {
    let segments: Vec<Vec<String>> = paths.iter().map(|path| segments_of(path)).collect();
    paths
        .into_iter()
        .zip(&segments)
        .enumerate()
        .map(|(i, (path, own))| {
            let shared = |len| {
                segments
                    .iter()
                    .enumerate()
                    .any(|(j, other)| j != i && suffix(other, len) == suffix(own, len))
            };
            let len = (1..own.len())
                .find(|&len| !shared(len))
                .unwrap_or(own.len());
            Member {
                name: suffix(own, len).join("/"),
                fork: is_fork(&path, fs),
                path,
            }
        })
        .collect()
}

/// A path's normal segments, as text.
fn segments_of(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(segment) => Some(segment.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}

/// The last `len` items of `segments`, or all of them when it holds fewer.
fn suffix(segments: &[String], len: usize) -> &[String] {
    segments
        .get(segments.len().saturating_sub(len)..)
        .unwrap_or_default()
}
