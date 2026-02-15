use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::Datelike;
use log::{debug, trace};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};

use crate::EvalError;

/// A minimal `World` implementation for evaluating `.typ` files.
///
/// This world resolves files from the local filesystem relative to a
/// project root directory. It caches loaded sources in memory and provides
/// an empty font book (eval-only, no layout/render).
///
/// It also tracks cross-file dependencies: whenever a file is loaded during
/// evaluation (via `source()` or `file()`), it records that dependency.
pub struct MindTapeWorld {
    /// Project root directory. All virtual paths resolve against this.
    root: PathBuf,
    /// The `FileId` of the main `.typ` file being evaluated.
    main_id: FileId,
    /// Standard Typst library.
    library: LazyHash<Library>,
    /// Empty font book (we never layout, so no fonts needed).
    book: LazyHash<FontBook>,
    /// Cache of loaded sources, keyed by `FileId`.
    sources: Mutex<HashMap<FileId, Source>>,
    /// Set of all file IDs accessed during evaluation (for dependency tracking).
    /// Excludes the main file itself.
    dependencies: Mutex<HashSet<FileId>>,
}

impl MindTapeWorld {
    /// Create a new world for evaluating the given `.typ` file.
    ///
    /// The file path may be relative (resolved against the current working
    /// directory). The project root is determined by walking up from the
    /// file's directory looking for project markers (`Cargo.toml`, `lib/`
    /// directory, or `.git`). If no marker is found, the file's parent
    /// directory is used as root.
    ///
    /// The main file's virtual path is computed relative to the discovered
    /// root so that relative imports (e.g. `../../lib/prelude.typ`) resolve
    /// correctly on disk.
    ///
    /// # Errors
    /// Returns `Err` if the file path cannot be resolved or canonicalized.
    pub fn new(file_path: &Path) -> Result<Self, EvalError> {
        let abs_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| EvalError::World(format!("failed to get cwd: {e}")))?
                .join(file_path)
        };

        let abs_path = abs_path
            .canonicalize()
            .map_err(|e| EvalError::World(format!("failed to canonicalize {}: {e}", file_path.display())))?;

        let file_dir = abs_path
            .parent()
            .ok_or_else(|| EvalError::World(format!("file has no parent directory: {}", abs_path.display())))?;

        let root = find_project_root(file_dir);

        let rel_path = abs_path
            .strip_prefix(&root)
            .map_err(|_| {
                EvalError::World(format!(
                    "file {} is not under project root {}",
                    abs_path.display(),
                    root.display()
                ))
            })?;

        debug!("project root: {}", root.display());
        trace!("main file: {}", rel_path.display());

        let vpath = VirtualPath::new(rel_path);
        let main_id = FileId::new(None, vpath);

        Ok(Self {
            root,
            main_id,
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(FontBook::new()),
            sources: Mutex::new(HashMap::new()),
            dependencies: Mutex::new(HashSet::new()),
        })
    }

    /// Resolve a `FileId` to an absolute filesystem path.
    ///
    /// For project-local files (`id.package()` is `None`), resolves against
    /// the project root. For `@mindtape` package files, resolves against
    /// `{root}/lib/` so that the package manifest and entry point map to
    /// `lib/typst.toml` and `lib/prelude.typ` respectively.
    ///
    /// For `@local/mindtape:VERSION` packages, resolves against
    /// `~/.local/share/typst/packages/local/mindtape/VERSION/` (following
    /// Typst's standard local package directory structure).
    fn resolve_path(&self, id: FileId) -> FileResult<PathBuf> {
        let root = match id.package() {
            Some(spec) if spec.namespace == "mindtape" => self.root.join("lib"),
            Some(spec) if spec.namespace == "local" && spec.name == "mindtape" => {
                // Resolve @local/mindtape:VERSION to ~/.local/share/typst/packages/local/mindtape/VERSION/
                let mut path = dirs::data_local_dir()
                    .ok_or_else(|| FileError::Other(Some("could not determine local data directory".into())))?;
                path.push("typst");
                path.push("packages");
                path.push("local");
                path.push(spec.name.as_str());
                path.push(spec.version.to_string());
                path
            }
            Some(_) => return Err(FileError::NotFound(id.vpath().as_rooted_path().into())),
            None => self.root.clone(),
        };
        id.vpath()
            .resolve(&root)
            .ok_or_else(|| FileError::AccessDenied)
    }

    /// Read and cache a source file.
    fn load_source(&self, id: FileId) -> FileResult<Source> {
        let path = self.resolve_path(id)?;
        let text = std::fs::read_to_string(&path).map_err(|e| FileError::from_io(e, &path))?;
        Ok(Source::new(id, text))
    }

    /// Record a file as a dependency (if it's not the main file).
    fn record_dependency(&self, id: FileId) {
        if id != self.main_id {
            let mut deps = self.dependencies.lock().expect("dependencies lock poisoned");
            deps.insert(id);
        }
    }

    /// Get all file dependencies discovered during evaluation.
    /// Returns relative paths (from project root) of files that were imported.
    ///
    /// # Errors
    /// Returns `Err` if a dependency's path cannot be resolved.
    ///
    /// # Panics
    /// Panics if the dependencies lock is poisoned (should never happen in normal operation).
    pub fn get_dependencies(&self) -> Result<Vec<PathBuf>, EvalError> {
        let deps = self.dependencies.lock().expect("dependencies lock poisoned");
        let mut paths = Vec::new();
        for id in deps.iter() {
            let abs_path = self.resolve_path(*id)
                .map_err(|e| EvalError::World(format!("failed to resolve dependency: {e}")))?;

            // Skip external dependencies (e.g. @local/mindtape packages) —
            // only track project-local files that might change.
            if let Ok(rel_path) = abs_path.strip_prefix(&self.root) {
                paths.push(rel_path.to_path_buf());
            } else {
                trace!(
                    "skipping external dependency: {}",
                    abs_path.display()
                );
            }
        }
        paths.sort();
        Ok(paths)
    }
}

impl typst::World for MindTapeWorld {
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
        // Record this file as a dependency (unless it's the main file).
        self.record_dependency(id);

        {
            let cache = self.sources.lock().expect("source cache poisoned");
            if let Some(source) = cache.get(&id) {
                trace!("source cache hit: {}", id.vpath().as_rooted_path().display());
                return Ok(source.clone());
            }
        }

        trace!("loading source: {}", id.vpath().as_rooted_path().display());
        let source = self.load_source(id)?;

        let mut cache = self.sources.lock().expect("source cache poisoned");
        cache.insert(id, source.clone());
        Ok(source)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        // Record this file as a dependency (unless it's the main file).
        self.record_dependency(id);

        let path = self.resolve_path(id)?;
        let data = std::fs::read(&path).map_err(|e| FileError::from_io(e, &path))?;
        Ok(Bytes::new(data))
    }

    fn font(&self, _index: usize) -> Option<Font> {
        None
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        let now = chrono::Local::now();
        #[allow(clippy::cast_possible_truncation)]
        {
            Datetime::from_ymd(now.year(), now.month() as u8, now.day() as u8)
        }
    }
}

/// Walk up from `start_dir` looking for project root markers.
///
/// Recognized markers (checked in order):
/// - `Cargo.toml` file
/// - `lib/` directory
/// - `.git` directory or file
///
/// Returns the first ancestor directory containing a marker, or `start_dir`
/// itself if no marker is found.
pub fn find_project_root(start_dir: &Path) -> PathBuf {
    let mut dir = start_dir.to_path_buf();
    loop {
        if dir.join("Cargo.toml").exists() {
            return dir;
        }
        if dir.join("lib").is_dir() {
            return dir;
        }
        if dir.join(".git").exists() {
            return dir;
        }
        if !dir.pop() {
            // Reached filesystem root without finding a marker.
            return start_dir.to_path_buf();
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use typst::World;

    // --- find_project_root ---

    #[test]
    fn find_project_root_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(find_project_root(dir.path()), dir.path());
    }

    #[test]
    fn find_project_root_git() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        assert_eq!(find_project_root(dir.path()), dir.path());
    }

    #[test]
    fn find_project_root_lib_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("lib")).unwrap();
        assert_eq!(find_project_root(dir.path()), dir.path());
    }

    #[test]
    fn find_project_root_walks_up() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let child = dir.path().join("sub").join("deep");
        std::fs::create_dir_all(&child).unwrap();
        assert_eq!(find_project_root(&child), dir.path());
    }

    #[test]
    fn find_project_root_no_marker_returns_nearest_ancestor_with_marker() {
        // On most systems, the walk will eventually find some marker (e.g. /lib).
        // What matters is that it never panics and returns a valid directory.
        let dir = tempfile::tempdir().unwrap();
        let child = dir.path().join("empty");
        std::fs::create_dir_all(&child).unwrap();
        let root = find_project_root(&child);
        assert!(root.is_dir());
    }

    #[test]
    fn find_project_root_cargo_toml_has_priority_over_git() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        // Cargo.toml is checked first, so it should match this dir
        assert_eq!(find_project_root(dir.path()), dir.path());
    }

    // --- MindTapeWorld ---

    #[test]
    fn world_new_with_valid_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let typ_file = dir.path().join("test.typ");
        std::fs::write(&typ_file, "= Hello").unwrap();

        let world = MindTapeWorld::new(&typ_file).unwrap();
        // Main file ID should be set
        let _main = world.main();
        // Should be able to load the source
        let source = world.source(world.main()).unwrap();
        assert!(source.text().contains("Hello"));
    }

    #[test]
    fn world_new_nonexistent_file() {
        let result = MindTapeWorld::new(Path::new("/nonexistent/path/test.typ"));
        assert!(result.is_err());
    }

    #[test]
    fn world_source_caching() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let typ_file = dir.path().join("test.typ");
        std::fs::write(&typ_file, "= Cached").unwrap();

        let world = MindTapeWorld::new(&typ_file).unwrap();
        let main_id = world.main();

        // First call loads from disk
        let s1 = world.source(main_id).unwrap();
        // Second call should return cached version
        let s2 = world.source(main_id).unwrap();
        assert_eq!(s1.text(), s2.text());
    }

    #[test]
    fn world_new_with_absolute_path() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let typ_file = dir.path().join("test.typ");
        std::fs::write(&typ_file, "= Abs").unwrap();

        // Pass the already-absolute path
        let abs = typ_file.canonicalize().unwrap();
        let world = MindTapeWorld::new(&abs).unwrap();
        let source = world.source(world.main()).unwrap();
        assert!(source.text().contains("Abs"));
    }

    #[test]
    fn world_file_reads_bytes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let typ_file = dir.path().join("test.typ");
        std::fs::write(&typ_file, "= Bytes Test").unwrap();

        let world = MindTapeWorld::new(&typ_file).unwrap();
        let bytes = world.file(world.main()).unwrap();
        assert!(std::str::from_utf8(bytes.as_slice()).unwrap().contains("Bytes Test"));
    }

    #[test]
    fn world_font_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let typ_file = dir.path().join("test.typ");
        std::fs::write(&typ_file, "").unwrap();

        let world = MindTapeWorld::new(&typ_file).unwrap();
        assert!(world.font(0).is_none());
    }

    #[test]
    fn world_today_returns_valid_date() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let typ_file = dir.path().join("test.typ");
        std::fs::write(&typ_file, "").unwrap();

        let world = MindTapeWorld::new(&typ_file).unwrap();
        let today = world.today(None);
        assert!(today.is_some());
        let dt = today.unwrap();
        assert!(dt.year().unwrap() >= 2024);
    }

    #[test]
    fn world_resolve_unknown_package_fails() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let typ_file = dir.path().join("test.typ");
        // Try to import from an unknown package — should fail during eval
        std::fs::write(&typ_file, r#"#import "@unknown/pkg:1.0.0": foo"#).unwrap();

        let world = MindTapeWorld::new(&typ_file).unwrap();
        let result = crate::eval_file(&world);
        assert!(result.is_err());
    }

    #[test]
    fn world_source_nonexistent_file_errors() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let typ_file = dir.path().join("test.typ");
        std::fs::write(&typ_file, "").unwrap();

        let world = MindTapeWorld::new(&typ_file).unwrap();
        // Create a FileId for a file that doesn't exist on disk
        let missing_id = FileId::new(None, VirtualPath::new("nonexistent.typ"));
        assert!(world.source(missing_id).is_err());
    }
}
