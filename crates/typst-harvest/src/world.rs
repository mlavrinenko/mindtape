//! A minimal, eval-only [`typst::World`] that resolves files from disk.
//!
//! [`HarvestWorld`] is package-agnostic: it resolves project-local imports
//! against a root directory and `@local/<name>:<version>` packages against the
//! standard Typst local-package directory. A caller may register in-repo
//! overrides (mapping a package name to a lib directory) so a tool developing
//! its own package resolves it from the working tree instead of an install.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::Datelike;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Scope};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};

use crate::HarvestError;

/// An eval-only Typst world backed by the local filesystem.
pub struct HarvestWorld {
    root: PathBuf,
    main_id: FileId,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    sources: Mutex<HashMap<FileId, Source>>,
    dependencies: Mutex<HashSet<FileId>>,
    /// Package name → absolute lib directory, for in-repo package development.
    local_overrides: HashMap<String, PathBuf>,
    /// File id → in-memory source text, consulted before any disk read. Lets a
    /// caller back a package entry point (or any file) with a synthesised source
    /// instead of an on-disk one — e.g. a generated DSL re-export module.
    source_overrides: HashMap<FileId, String>,
}

impl HarvestWorld {
    /// Build a world for `file_path`, auto-detecting the project root by walking
    /// up for a generic `Cargo.toml`, `lib/`, or `.git` marker. A tool with its
    /// own root convention should use [`Self::with_root`] instead.
    ///
    /// # Errors
    /// Returns an error if the path cannot be resolved or canonicalized.
    pub fn new(file_path: &Path) -> Result<Self, HarvestError> {
        let abs = absolutize(file_path)?;
        let dir = parent_dir(&abs)?;
        let root = find_project_root(dir);
        Self::assemble(&abs, root)
    }

    /// Build a world for `file_path` with an explicit project `root`.
    ///
    /// # Errors
    /// Returns an error if the path cannot be resolved or lies outside `root`.
    pub fn with_root(file_path: &Path, root: &Path) -> Result<Self, HarvestError> {
        let abs = absolutize(file_path)?;
        let root = root
            .canonicalize()
            .map_err(|err| HarvestError::World(format!("bad root {}: {err}", root.display())))?;
        Self::assemble(&abs, root)
    }

    /// Register an in-repo lib directory for package `name` (builder style).
    #[must_use]
    pub fn with_local_package(mut self, name: &str, lib_dir: PathBuf) -> Self {
        self.local_overrides.insert(name.to_owned(), lib_dir);
        self
    }

    /// Back `id` with an in-memory `source`, consulted before any disk read.
    ///
    /// Use this to inject a synthesised module — e.g. a package entry point that
    /// re-exports natively-bound DSL functions — so the file never needs to exist
    /// on disk. The package manifest (`typst.toml`) is still read from disk to
    /// learn the entry point, so pair this with [`Self::with_local_package`] when
    /// overriding a package's entry source.
    #[must_use]
    pub fn with_source_override(mut self, id: FileId, source: impl Into<String>) -> Self {
        self.source_overrides.insert(id, source.into());
        self
    }

    /// Merge `scope`'s bindings into the evaluation library's global scope, so
    /// the document can call host-native functions (a DSL bound in Rust) without
    /// importing them. Later bindings win over the defaults.
    #[must_use]
    pub fn with_globals(mut self, scope: &Scope) -> Self {
        let global = self.library.global.scope_mut();
        for (name, binding) in scope.iter() {
            global.bind(name.clone(), binding.clone());
        }
        self
    }

    fn assemble(abs: &Path, root: PathBuf) -> Result<Self, HarvestError> {
        let rel = abs.strip_prefix(&root).map_err(|_| {
            HarvestError::World(format!(
                "file {} is not under project root {}",
                abs.display(),
                root.display()
            ))
        })?;
        let main_id = FileId::new(None, VirtualPath::new(rel));
        Ok(Self {
            root,
            main_id,
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(FontBook::new()),
            sources: Mutex::new(HashMap::new()),
            dependencies: Mutex::new(HashSet::new()),
            local_overrides: HashMap::new(),
            source_overrides: HashMap::new(),
        })
    }

    /// The project root all virtual paths resolve against.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn resolve_path(&self, id: FileId) -> FileResult<PathBuf> {
        let base = match id.package() {
            Some(spec) => self.resolve_package(spec)?,
            None => self.root.clone(),
        };
        id.vpath().resolve(&base).ok_or(FileError::AccessDenied)
    }

    fn resolve_package(&self, spec: &typst::syntax::package::PackageSpec) -> FileResult<PathBuf> {
        if let Some(dir) = self.local_overrides.get(spec.name.as_str()) {
            return Ok(dir.clone());
        }
        if spec.namespace == "local" {
            let mut path = dirs::data_local_dir().ok_or_else(|| {
                FileError::Other(Some("could not determine local data dir".into()))
            })?;
            path.push("typst");
            path.push("packages");
            path.push("local");
            path.push(spec.name.as_str());
            path.push(spec.version.to_string());
            return Ok(path);
        }
        Err(FileError::NotFound(id_vpath(spec)))
    }

    fn record_dependency(&self, id: FileId) {
        if id != self.main_id
            && let Ok(mut deps) = self.dependencies.lock()
        {
            deps.insert(id);
        }
    }

    /// Project-local files read during evaluation, relative to the root.
    ///
    /// # Errors
    /// Returns an error if a dependency path cannot be resolved.
    pub fn dependencies(&self) -> Result<Vec<PathBuf>, HarvestError> {
        let deps = self
            .dependencies
            .lock()
            .map_err(|_| HarvestError::World("dependencies lock poisoned".to_owned()))?;
        let mut paths = Vec::new();
        for id in deps.iter() {
            let abs = self
                .resolve_path(*id)
                .map_err(|err| HarvestError::World(format!("resolve dependency: {err}")))?;
            if let Ok(rel) = abs.strip_prefix(&self.root) {
                paths.push(rel.to_path_buf());
            }
        }
        paths.sort();
        Ok(paths)
    }
}

impl typst::World for HarvestWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main_id
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.record_dependency(id);
        if let Ok(cache) = self.sources.lock()
            && let Some(source) = cache.get(&id)
        {
            return Ok(source.clone());
        }
        let text = if let Some(synthetic) = self.source_overrides.get(&id) {
            synthetic.clone()
        } else {
            let path = self.resolve_path(id)?;
            std::fs::read_to_string(&path).map_err(|err| FileError::from_io(err, &path))?
        };
        let source = Source::new(id, text);
        if let Ok(mut cache) = self.sources.lock() {
            cache.insert(id, source.clone());
        }
        Ok(source)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.record_dependency(id);
        let path = self.resolve_path(id)?;
        let data = std::fs::read(&path).map_err(|err| FileError::from_io(err, &path))?;
        Ok(Bytes::new(data))
    }

    fn font(&self, _index: usize) -> Option<Font> {
        None
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        let now = chrono::Local::now();
        Datetime::from_ymd(
            now.year(),
            u8::try_from(now.month()).unwrap_or(1),
            u8::try_from(now.day()).unwrap_or(1),
        )
    }
}

fn id_vpath(spec: &typst::syntax::package::PackageSpec) -> PathBuf {
    PathBuf::from(format!(
        "@{}/{}:{}",
        spec.namespace, spec.name, spec.version
    ))
}

fn absolutize(file_path: &Path) -> Result<PathBuf, HarvestError> {
    let abs = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|err| HarvestError::World(format!("cwd: {err}")))?
            .join(file_path)
    };
    abs.canonicalize()
        .map_err(|err| HarvestError::World(format!("canonicalize {}: {err}", file_path.display())))
}

fn parent_dir(abs: &Path) -> Result<&Path, HarvestError> {
    abs.parent()
        .ok_or_else(|| HarvestError::World(format!("no parent dir: {}", abs.display())))
}

/// Walk up from `start` for a generic project-root marker, in priority order:
/// `Cargo.toml`, `lib/`, `.git`. Falls back to `start`. Consumer-blind: a tool
/// with its own root convention (e.g. a dotdir) should locate the root itself
/// and pass it to [`HarvestWorld::with_root`].
#[must_use]
pub fn find_project_root(start: &Path) -> PathBuf {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("Cargo.toml").is_file() || dir.join("lib").is_dir() || dir.join(".git").exists()
        {
            return dir;
        }
        if !dir.pop() {
            return start.to_path_buf();
        }
    }
}

#[cfg(test)]
#[path = "world_tests.rs"]
mod tests;
