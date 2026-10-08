//! Stem hashes: the short code an agent types to address a task.
//!
//! A stem's code is FNV-1a 64 of its UTF-8 bytes, run through the splitmix64
//! finaliser, in lowercase Crockford base32, thirteen digits, most significant first, the first twelve
//! digits five bits each from the top. Its stem hash is the
//! shortest prefix of that code no other stem's code shares, at least
//! [`MIN_LEN`] digits. A new stem therefore lengthens only the hashes that
//! share a prefix with it: an older short hash turns ambiguous, and never
//! resolves to another task. A hash never equals a stem or a reserved word: it lengthens past one. Stems with equal codes stay ambiguous at every
//! length. Folders do not enter the code.

use std::path::{Path, PathBuf};

use crate::model::Task;

/// The fewest digits a stem hash has, and the fewest an input may have.
pub const MIN_LEN: usize = 3;

const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";
const CODE_LEN: usize = 13;

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
    /// A path that exists, but that no loaded task owns.
    #[error("no task at {}", .0.display())]
    NoTaskAt(PathBuf),
    /// A stem hash several tasks share.
    #[error("stem hash {reference:?} names {} tasks", .stems.len())]
    Ambiguous {
        /// The typed reference.
        reference: String,
        /// The stems it names, sorted.
        stems: Vec<String>,
    },
    /// The reference is empty, so it names nothing to look for.
    #[error("empty task reference")]
    Empty,
    /// Neither a stem nor a stem hash matches.
    #[error("no task with id {0:?}")]
    NoMatch(String),
}

/// The task a reference names, and its file.
#[derive(Debug, Clone)]
pub struct Resolved<'a> {
    /// The task.
    pub task: &'a Task,
    /// Its file: canonical when the reference was a path, else under the root.
    pub path: PathBuf,
}

/// Whether `reference` is spelled as a path: absolute, holding a separator, or
/// ending `.typ`. Anything else is a stem or a stem hash.
#[must_use]
pub fn is_path_ref(reference: &str) -> bool {
    Path::new(reference).is_absolute()
        || reference.contains(['/', '\\'])
        || reference.ends_with(".typ")
}

/// Resolves `reference` to one of `tasks`. A path resolves from `cwd`; else an
/// exact stem wins, and a stem hash from `hashes` is tried only when no stem
/// matches. Task files are `root`-relative.
///
/// # Errors
/// Returns a [`RefError`] when the reference is empty, a path does not resolve
/// or owns no task, a hash is ambiguous, or nothing matches.
pub fn resolve_ref<'a>(
    tasks: &'a [Task],
    root: &Path,
    cwd: &Path,
    hashes: &StemHashes,
    reference: &str,
) -> Result<Resolved<'a>, RefError> {
    if reference.is_empty() {
        return Err(RefError::Empty);
    }
    if is_path_ref(reference) {
        return resolve_path(tasks, root, cwd, reference);
    }
    let by_stem = |stem: &str| tasks.iter().find(|task| task.id == stem);
    let task = match by_stem(reference) {
        Some(task) => task,
        None => match hashes.resolve(reference) {
            Resolution::Found(stem) => by_stem(&stem),
            Resolution::Ambiguous(stems) => {
                return Err(RefError::Ambiguous {
                    reference: reference.to_owned(),
                    stems,
                });
            }
            Resolution::NoMatch => None,
        }
        .ok_or_else(|| RefError::NoMatch(reference.to_owned()))?,
    };
    Ok(Resolved {
        task,
        path: root.join(&task.file),
    })
}

fn resolve_path<'a>(
    tasks: &'a [Task],
    root: &Path,
    cwd: &Path,
    reference: &str,
) -> Result<Resolved<'a>, RefError> {
    let joined = cwd.join(reference);
    let abs = joined.canonicalize().map_err(|source| RefError::Path {
        path: joined,
        source,
    })?;
    let task = tasks
        .iter()
        .find(|task| root.join(&task.file).canonicalize().ok().as_deref() == Some(&abs))
        .ok_or_else(|| RefError::NoTaskAt(abs.clone()))?;
    Ok(Resolved { task, path: abs })
}

/// What a typed stem hash names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Exactly one stem matches.
    Found(String),
    /// Several stems match; their stems, sorted.
    Ambiguous(Vec<String>),
    /// No stem matches, or the input is too short or not a code.
    NoMatch,
}

/// The stem hash of each stem in a set.
#[derive(Debug, Clone)]
pub struct StemHashes {
    entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
struct Entry {
    stem: String,
    code: String,
    /// The digits of `code` that make the stem hash.
    len: usize,
}

impl StemHashes {
    /// Hashes every stem in `stems`. No hash equals any of the stems.
    pub fn new<'a>(stems: impl IntoIterator<Item = &'a str>) -> Self {
        Self::reserving(stems, |_| false)
    }

    /// Hashes every stem in `stems`. A hash equals neither a stem nor a word
    /// `reserved` accepts: it lengthens past it, so a typed reference never
    /// reads as the stem or the word instead of the task.
    pub fn reserving<'a>(
        stems: impl IntoIterator<Item = &'a str>,
        reserved: impl Fn(&str) -> bool,
    ) -> Self {
        Self::build(stems, |stem| mix(fnv1a_64(stem.as_bytes())), reserved)
    }

    /// Hashes every stem with `hash` in place of mixed FNV-1a 64, so a test can
    /// force a shared prefix or a full collision.
    pub fn with_hasher<'a>(
        stems: impl IntoIterator<Item = &'a str>,
        hash: impl Fn(&str) -> u64,
    ) -> Self {
        Self::build(stems, hash, |_| false)
    }

    fn build<'a>(
        stems: impl IntoIterator<Item = &'a str>,
        hash: impl Fn(&str) -> u64,
        reserved: impl Fn(&str) -> bool,
    ) -> Self {
        let stems: std::collections::HashSet<&str> = stems.into_iter().collect();
        let mut entries: Vec<Entry> = stems
            .iter()
            .map(|&stem| Entry {
                stem: stem.to_owned(),
                code: encode(hash(stem)),
                len: MIN_LEN,
            })
            .collect();
        // In code order a code's longest shared prefix is with a neighbour.
        entries.sort_by(|a, b| a.code.cmp(&b.code).then_with(|| a.stem.cmp(&b.stem)));
        let gaps: Vec<usize> = entries
            .windows(2)
            .map(|pair| match pair {
                [a, b] => common_prefix(&a.code, &b.code),
                _ => 0,
            })
            .collect();
        let before = std::iter::once(0).chain(gaps.iter().copied());
        let after = gaps.iter().copied().chain(std::iter::once(0));
        for (entry, (before, after)) in entries.iter_mut().zip(before.zip(after)) {
            entry.len = (before.max(after) + 1).clamp(MIN_LEN, CODE_LEN);
            while entry.len < CODE_LEN
                && entry
                    .code
                    .get(..entry.len)
                    .is_some_and(|hash| stems.contains(hash) || reserved(hash))
            {
                entry.len += 1;
            }
        }
        entries.sort_by(|a, b| a.stem.cmp(&b.stem));
        Self { entries }
    }

    /// The stem hash of `stem`, or `None` when `stem` is not in the set.
    #[must_use]
    pub fn hash_of(&self, stem: &str) -> Option<&str> {
        let at = self
            .entries
            .binary_search_by(|entry| entry.stem.as_str().cmp(stem))
            .ok()?;
        let entry = self.entries.get(at)?;
        entry.code.get(..entry.len)
    }

    /// Whether the digits typed so far, `typed`, begin the code of `stem`. They
    /// read as [`resolve`](Self::resolve) reads them, whatever their count, so
    /// a caller narrowing a list as keys arrive agrees with it. A stem not in
    /// the set, or a character outside the alphabet, matches nothing.
    #[must_use]
    pub fn matches(&self, stem: &str, typed: &str) -> bool {
        let typed = normalise_digits(typed);
        self.entries
            .binary_search_by(|entry| entry.stem.as_str().cmp(stem))
            .ok()
            .and_then(|at| self.entries.get(at))
            .is_some_and(|entry| entry.code.starts_with(&typed))
    }

    /// Resolves typed `input` to a stem. Input is case-insensitive, reads `i`
    /// and `l` as `1` and `o` as `0`, and needs at least [`MIN_LEN`] digits.
    #[must_use]
    pub fn resolve(&self, input: &str) -> Resolution {
        let Some(prefix) = normalise(input) else {
            return Resolution::NoMatch;
        };
        let mut hits = self
            .entries
            .iter()
            .filter(|entry| entry.code.starts_with(&prefix))
            .map(|entry| entry.stem.clone());
        match (hits.next(), hits.next()) {
            (None, _) => Resolution::NoMatch,
            (Some(stem), None) => Resolution::Found(stem),
            (Some(first), Some(second)) => {
                Resolution::Ambiguous([first, second].into_iter().chain(hits).collect())
            }
        }
    }
}

/// FNV-1a 64 of `bytes`.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// The splitmix64 finaliser: spreads every input bit through all output bits,
/// so stems that differ only at the end do not share leading digits.
fn mix(value: u64) -> u64 {
    let step = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let step = (step ^ (step >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    step ^ (step >> 31)
}

/// `value` as thirteen Crockford base32 digits, most significant first. The
/// first twelve take five bits each from the top and the last takes the
/// remaining four, so every leading digit carries a full five bits.
fn encode(value: u64) -> String {
    let aligned = u128::from(value) << 1;
    (0..CODE_LEN)
        .rev()
        .map(|digit| {
            let index = usize::try_from((aligned >> (5 * digit)) & 31).unwrap_or(0);
            ALPHABET.get(index).map_or('0', |&byte| char::from(byte))
        })
        .collect()
}

/// `input` lowercased, with `i` and `l` read as `1` and `o` as `0`.
fn normalise_digits(input: &str) -> String {
    input
        .chars()
        .map(|c| match c.to_ascii_lowercase() {
            'i' | 'l' => '1',
            'o' => '0',
            other => other,
        })
        .collect()
}

/// Typed input as a code prefix, or `None` when it is too short or holds a
/// character outside the alphabet.
fn normalise(input: &str) -> Option<String> {
    let prefix = normalise_digits(input);
    let valid = prefix
        .chars()
        .all(|c| c.is_ascii() && ALPHABET.contains(&(c as u8)));
    (valid && prefix.len() >= MIN_LEN).then_some(prefix)
}

/// The count of leading characters `a` and `b` share.
fn common_prefix(a: &str, b: &str) -> usize {
    a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count()
}

#[cfg(test)]
#[path = "handle_tests.rs"]
mod tests;
