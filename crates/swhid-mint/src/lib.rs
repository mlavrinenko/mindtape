//! Mint and resolve Software Heritage identifiers (SWHID, ISO/IEC 18670:2025).
//!
//! The core — [`Swhid`], [`Swhid::render`], [`Swhid::parse`] — is VCS-agnostic: a
//! SWHID is `swh:1:<kind>:<hash>` with optional qualifiers. Deriving an id from a
//! working tree, or checking one still resolves, is a VCS-backed operation behind
//! a backend feature: [`git`] today (via the `gix` crate), other VCS later.
//!
//! Why hand-rolled, not the `swhid` crate: for content/directory/revision the
//! SWHID hash is `sha1_git` — identical to git's own blob/tree/commit SHA-1 — so a
//! backend reads the hash straight from the VCS and we only prefix
//! `swh:1:<kind>:`. The `swhid` crate's `git` feature pulls libgit2 and targets
//! archival ids of HEAD/tags/snapshots; its core drags `clap` and a SHA-1
//! implementation this crate never needs.
//!
//! ```
//! use swhid_mint::{Kind, Swhid};
//! let text = "swh:1:rev:0000000000000000000000000000000000000000;path=a/b.typ";
//! let id = Swhid::parse(text).expect("valid");
//! assert_eq!(id.kind, Kind::Revision);
//! assert_eq!(id.path.as_deref(), Some("a/b.typ"));
//! assert_eq!(id.render(), text);
//! ```

#[cfg(feature = "git")]
pub mod git;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

/// The object a SWHID names. v1 hashes are `sha1_git`, identical to git's own
/// blob/tree/commit SHA-1 for content/directory/revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// File content (`cnt`) — a git blob.
    Content,
    /// Directory (`dir`) — a git tree.
    Directory,
    /// Revision (`rev`) — a git commit.
    Revision,
    /// Release (`rel`) — a git annotated tag.
    Release,
    /// Snapshot (`snp`).
    Snapshot,
}

impl Kind {
    /// The three-letter SWHID type code (`cnt`, `dir`, `rev`, `rel`, `snp`).
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::Content => "cnt",
            Self::Directory => "dir",
            Self::Revision => "rev",
            Self::Release => "rel",
            Self::Snapshot => "snp",
        }
    }

    fn from_code(code: &str) -> Option<Self> {
        match code {
            "cnt" => Some(Self::Content),
            "dir" => Some(Self::Directory),
            "rev" => Some(Self::Revision),
            "rel" => Some(Self::Release),
            "snp" => Some(Self::Snapshot),
            _ => None,
        }
    }
}

/// A SWHID core identifier plus the `path` qualifier this crate uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Swhid {
    /// The object kind.
    pub kind: Kind,
    /// 40-character lowercase hex `sha1_git`.
    pub hash: String,
    /// Optional `;path=` qualifier (repository-relative).
    pub path: Option<String>,
}

impl Swhid {
    /// Render as `swh:1:<kind>:<hash>[;path=<path>]`.
    #[must_use]
    pub fn render(&self) -> String {
        let core = format!("swh:1:{}:{}", self.kind.code(), self.hash);
        match &self.path {
            Some(path) => format!("{core};path={path}"),
            None => core,
        }
    }

    /// Parse `swh:1:<kind>:<40-hex>[;qualifiers]`. Returns `None` on a wrong
    /// version, an unknown kind, a malformed hash, or an unexpected shape. Only
    /// the `path` qualifier is retained; others are ignored.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let rest = text.strip_prefix("swh:1:")?;
        let (body, qualifiers) = rest.split_once(';').unwrap_or((rest, ""));
        let (code, hash) = body.split_once(':')?;
        let kind = Kind::from_code(code)?;
        if !is_sha1_git(hash) {
            return None;
        }
        Some(Self {
            kind,
            hash: hash.to_owned(),
            path: path_qualifier(qualifiers),
        })
    }
}

/// Cheap shape check: does `url` use the SWHID scheme (`swh:1:`)? A prefix test,
/// not a full parse — callers use it to tell a SWHID tombstone URL apart from a
/// file path or an http link without paying for [`Swhid::parse`]. Co-located with
/// the parser that owns the scheme.
#[must_use]
pub fn is_swhid_url(url: &str) -> bool {
    url.starts_with("swh:1:")
}

fn is_sha1_git(hash: &str) -> bool {
    hash.len() == 40
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn path_qualifier(qualifiers: &str) -> Option<String> {
    qualifiers
        .split(';')
        .find_map(|q| q.strip_prefix("path="))
        .map(ToOwned::to_owned)
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
