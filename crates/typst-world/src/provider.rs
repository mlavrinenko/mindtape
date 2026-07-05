//! The file-provider seam: how a [`crate::World`] turns a [`FileId`] into bytes.
//!
//! Keeping byte reads behind a trait lets one world serve every target. Native
//! resolves and reads from disk ([`DiskProvider`]); a wasm build supplies a
//! provider that keys embedded bytes off the [`FileId`] directly, ignoring the
//! disk-oriented `root`/`overrides` it is handed.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use typst::diag::{FileError, FileResult};
use typst::foundations::Bytes;
use typst::syntax::package::{PackageSpec, PackageVersion};
use typst::syntax::{FileId, RootedPath, VirtualPath, VirtualRoot};

use crate::WorldError;

/// The resolution context a [`FileProvider`] is given alongside a [`FileId`]:
/// the project root for tree-local ids and the in-repo package overrides.
pub struct ProviderCtx<'a> {
    /// Project root that ids with no package resolve against.
    pub root: &'a Path,
    /// Package name → in-repo lib directory, consulted before the platform
    /// local-package directory.
    pub overrides: &'a HashMap<String, PathBuf>,
}

/// Reads the raw bytes backing a [`FileId`]. The world caches and tracks
/// dependencies above this seam, so an implementation only resolves and reads.
///
/// `Send + Sync` because [`typst::World`] requires it; a provider is shared
/// across the (single-threaded, on wasm) evaluation by reference.
pub trait FileProvider: Send + Sync {
    /// Read the bytes for `id`. A disk provider resolves the id to a path under
    /// `ctx.root` (or a package directory) and reads it; a bytes-backed provider
    /// may ignore `ctx` and look `id` up directly.
    ///
    /// # Errors
    /// Returns a [`FileError`] if the id cannot be resolved or read.
    fn read(&self, id: FileId, ctx: &ProviderCtx<'_>) -> FileResult<Bytes>;
}

/// A [`FileProvider`] that resolves ids to filesystem paths and reads them.
#[derive(Debug, Clone, Copy, Default)]
pub struct DiskProvider;

impl FileProvider for DiskProvider {
    fn read(&self, id: FileId, ctx: &ProviderCtx<'_>) -> FileResult<Bytes> {
        let path = resolve_path(id, ctx)?;
        let data = std::fs::read(&path).map_err(|err| FileError::from_io(err, &path))?;
        Ok(Bytes::new(data))
    }
}

/// A [`FileProvider`] backed by an in-memory id→bytes map — the web target's
/// answer to [`DiskProvider`]. It ignores the disk-oriented `ctx` entirely and
/// serves each [`FileId`] from the map, so sources, a `@local` package, and any
/// data file all come from embedded bytes. Build the ids with [`root_file_id`]
/// and [`local_package_file_id`].
#[derive(Debug, Default)]
pub struct BytesProvider {
    files: HashMap<FileId, Bytes>,
}

impl BytesProvider {
    /// An empty provider.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register `bytes` under `id` (builder form).
    #[must_use]
    pub fn with_file(mut self, id: FileId, bytes: impl Into<Vec<u8>>) -> Self {
        self.files.insert(id, Bytes::new(bytes.into()));
        self
    }
}

impl FileProvider for BytesProvider {
    fn read(&self, id: FileId, _ctx: &ProviderCtx<'_>) -> FileResult<Bytes> {
        self.files
            .get(&id)
            .cloned()
            .ok_or_else(|| FileError::NotFound(PathBuf::from(id.vpath().get_without_slash())))
    }
}

/// The [`FileId`] of a project-local (package-less) file at `vpath`, e.g. the
/// main document source a [`BytesProvider`] serves.
///
/// # Errors
/// Returns [`WorldError`] if `vpath` is not a valid virtual path.
pub fn root_file_id(vpath: &str) -> Result<FileId, WorldError> {
    let vpath = VirtualPath::new(vpath)
        .map_err(|err| WorldError::World(format!("bad virtual path {vpath:?}: {err}")))?;
    Ok(FileId::new(RootedPath::new(VirtualRoot::Project, vpath)))
}

/// The [`FileId`] of `vpath` inside the `@local/<name>:<version>` package — the
/// manifest (`typst.toml`) and entrypoint a document's import resolves to.
///
/// # Errors
/// Returns [`WorldError`] if `version` is not a valid `major.minor.patch` or
/// `vpath` is not a valid virtual path.
pub fn local_package_file_id(name: &str, version: &str, vpath: &str) -> Result<FileId, WorldError> {
    let version: PackageVersion = version
        .parse()
        .map_err(|err| WorldError::World(format!("bad package version {version:?}: {err}")))?;
    let spec = PackageSpec {
        namespace: "local".into(),
        name: name.into(),
        version,
    };
    let vpath = VirtualPath::new(vpath)
        .map_err(|err| WorldError::World(format!("bad virtual path {vpath:?}: {err}")))?;
    Ok(FileId::new(RootedPath::new(
        VirtualRoot::Package(spec),
        vpath,
    )))
}

/// Resolve a [`FileId`] to an absolute filesystem path: project-local ids
/// against `ctx.root`, package ids against an in-repo override or the platform
/// local-package directory.
///
/// # Errors
/// Returns a [`FileError`] if the path escapes its base or a package namespace
/// other than `local` (with no override) is requested.
pub fn resolve_path(id: FileId, ctx: &ProviderCtx<'_>) -> FileResult<PathBuf> {
    let base = match id.root() {
        VirtualRoot::Package(spec) => resolve_package(spec, ctx.overrides)?,
        VirtualRoot::Project => ctx.root.to_path_buf(),
    };
    id.vpath()
        .realize(&base)
        .map_err(|_| FileError::AccessDenied)
}

fn resolve_package(
    spec: &PackageSpec,
    overrides: &HashMap<String, PathBuf>,
) -> FileResult<PathBuf> {
    if let Some(dir) = overrides.get(spec.name.as_str()) {
        return Ok(dir.clone());
    }
    if spec.namespace == "local" {
        let mut path = dirs::data_local_dir()
            .ok_or_else(|| FileError::Other(Some("could not determine local data dir".into())))?;
        path.push("typst");
        path.push("packages");
        path.push("local");
        path.push(spec.name.as_str());
        path.push(spec.version.to_string());
        return Ok(path);
    }
    Err(FileError::NotFound(PathBuf::from(format!(
        "@{}/{}:{}",
        spec.namespace, spec.name, spec.version
    ))))
}
