//! The `@local/mindtape` packages this crate embeds, and the file provider
//! that serves them where no package directory holds that version, so a CI
//! runner or a fresh clone loads tasks before anyone runs `mt install`.
//!
//! Each supported version keeps its own immutable snapshot under
//! `preludes/<version>/`. A release archives the outgoing current snapshot
//! there before it bumps `lib/typst.toml`; a version leaves the set only with
//! its snapshot and fixture, in the release that stops reading it.

use std::path::PathBuf;
use std::sync::LazyLock;

use typst::foundations::Bytes;
use typst::syntax::FileId;
use typst::syntax::package::PackageVersion;
use typst_world::{
    BytesProvider, FallbackProvider, FileProvider, ScopedDiskProvider, local_package_file_id,
};

/// The prelude package's current version, and the highest this binary reads.
/// Mirrors `lib/typst.toml`.
pub const PRELUDE_VERSION: &str = "0.4.0";

/// Every prelude version this binary evaluates, oldest first. The current
/// version is the last entry.
pub const PRELUDE_SUPPORTED: &[&str] = &["0.2.0", "0.3.0", "0.4.0"];

/// The current prelude, a copy of `lib/prelude.typ` inside the crate so the
/// published tarball holds it; a test keeps the two equal.
pub const PRELUDE: &str = include_str!("preludes/0.4.0/prelude.typ");

const PRELUDE_0_2: &str = include_str!("preludes/0.2.0/prelude.typ");
const PRELUDE_0_3: &str = include_str!("preludes/0.3.0/prelude.typ");

/// One supported prelude version and the source it ships.
#[derive(Debug, Clone, Copy)]
pub struct PreludeSnapshot {
    /// The version, as `lib/typst.toml` spells it.
    pub version: &'static str,
    /// The prelude source at that version.
    pub source: &'static str,
}

/// The snapshot of every supported version, oldest first. Immutable: a
/// release copies the outgoing current here and never edits an entry.
pub const PRELUDE_SNAPSHOTS: &[PreludeSnapshot] = &[
    PreludeSnapshot {
        version: "0.2.0",
        source: PRELUDE_0_2,
    },
    PreludeSnapshot {
        version: "0.3.0",
        source: PRELUDE_0_3,
    },
    PreludeSnapshot {
        version: PRELUDE_VERSION,
        source: PRELUDE,
    },
];

impl PreludeSnapshot {
    /// The prelude package's `typst.toml` at this version.
    #[must_use]
    pub fn manifest(&self) -> String {
        format!(
            "[package]\nname = \"mindtape\"\nversion = \"{}\"\nentrypoint = \"prelude.typ\"\n",
            self.version
        )
    }
}

/// The snapshot of `version`, compared as an exact parsed
/// [`PackageVersion`], or `None` when this binary does not support it.
#[must_use]
pub fn prelude_snapshot(version: &str) -> Option<&'static PreludeSnapshot> {
    let wanted = version.parse::<PackageVersion>().ok()?;
    PRELUDE_SNAPSHOTS.iter().find(|snapshot| {
        snapshot
            .version
            .parse::<PackageVersion>()
            .is_ok_and(|found| found == wanted)
    })
}

/// Whether this binary evaluates the prelude at `version`.
#[must_use]
pub fn prelude_supports(version: &str) -> bool {
    prelude_snapshot(version).is_some()
}

/// Whether `version` parses and is older than [`PRELUDE_VERSION`]. A newer
/// version reads false, which is what picks the upgrade hint over the rewrite
/// hint.
#[must_use]
pub fn prelude_is_older(version: &str) -> bool {
    let (Ok(found), Ok(current)) = (
        version.parse::<PackageVersion>(),
        PRELUDE_VERSION.parse::<PackageVersion>(),
    ) else {
        return false;
    };
    found < current
}

/// The prelude package's `typst.toml` at the current version.
#[must_use]
pub fn prelude_manifest() -> String {
    format!(
        "[package]\nname = \"mindtape\"\nversion = \"{PRELUDE_VERSION}\"\nentrypoint = \"prelude.typ\"\n"
    )
}

/// The embedded packages' files, built once: Typst hashes each `Bytes` copy,
/// and every task file's world reads them.
static EMBEDDED: LazyLock<Vec<(FileId, Bytes)>> = LazyLock::new(|| {
    let mut files = Vec::new();
    for snapshot in PRELUDE_SNAPSHOTS {
        for (vpath, bytes) in [
            ("typst.toml", snapshot.manifest().into_bytes()),
            ("prelude.typ", snapshot.source.as_bytes().to_vec()),
        ] {
            if let Ok(id) = local_package_file_id("mindtape", snapshot.version, vpath) {
                files.push((id, Bytes::new(bytes)));
            }
        }
    }
    files
});

/// Reads `@local` packages under `package_root`, as Typst does, and an
/// embedded prelude where that directory lacks it. An installed copy wins, so
/// `mt` and an editor read the same file once `mt install` ran.
pub(crate) fn package_provider(package_root: Option<PathBuf>) -> Box<dyn FileProvider> {
    let embedded = EMBEDDED
        .iter()
        .fold(BytesProvider::new(), |provider, (id, bytes)| {
            provider.with_bytes(*id, bytes.clone())
        });
    Box::new(FallbackProvider::new(
        ScopedDiskProvider::new(package_root),
        embedded,
    ))
}

#[cfg(test)]
#[path = "embedded_tests.rs"]
mod tests;
