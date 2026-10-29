//! The `@local/mindtape` package this crate embeds, and the file provider that
//! serves it where no package directory holds that version, so a CI runner or
//! a fresh clone loads tasks before anyone runs `mt install`.

use std::path::PathBuf;
use std::sync::LazyLock;

use typst::foundations::Bytes;
use typst::syntax::FileId;
use typst_world::{
    BytesProvider, FallbackProvider, FileProvider, ScopedDiskProvider, local_package_file_id,
};

/// The prelude package's version. Mirrors `lib/typst.toml`.
pub const PRELUDE_VERSION: &str = "0.2.0";

/// A copy of `lib/prelude.typ`, inside the crate so the published tarball
/// holds it; a test keeps the two equal.
pub const PRELUDE: &str = include_str!("prelude.typ");

/// The prelude package's `typst.toml`.
#[must_use]
pub fn prelude_manifest() -> String {
    format!(
        "[package]\nname = \"mindtape\"\nversion = \"{PRELUDE_VERSION}\"\nentrypoint = \"prelude.typ\"\n"
    )
}

/// The embedded package's files, built once: Typst hashes each `Bytes` copy,
/// and every task file's world reads them.
static EMBEDDED: LazyLock<Vec<(FileId, Bytes)>> = LazyLock::new(|| {
    [
        ("typst.toml", prelude_manifest().into_bytes()),
        ("prelude.typ", PRELUDE.as_bytes().to_vec()),
    ]
    .into_iter()
    .filter_map(|(vpath, bytes)| {
        local_package_file_id("mindtape", PRELUDE_VERSION, vpath)
            .ok()
            .map(|id| (id, Bytes::new(bytes)))
    })
    .collect()
});

/// Reads `@local` packages under `package_root`, as Typst does, and the
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
