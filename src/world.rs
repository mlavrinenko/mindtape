use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};

/// A minimal `World` implementation for evaluating `.typ` files.
///
/// This world resolves files from the local filesystem relative to a
/// project root directory. It caches loaded sources in memory and provides
/// an empty font book (eval-only, no layout/render).
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
    pub fn new(file_path: &Path) -> Result<Self, String> {
        let abs_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| format!("failed to get cwd: {e}"))?
                .join(file_path)
        };

        let abs_path = abs_path
            .canonicalize()
            .map_err(|e| format!("failed to canonicalize {}: {e}", file_path.display()))?;

        let file_dir = abs_path
            .parent()
            .ok_or_else(|| format!("file has no parent directory: {}", abs_path.display()))?;

        let root = find_project_root(file_dir);

        let rel_path = abs_path
            .strip_prefix(&root)
            .map_err(|_| {
                format!(
                    "file {} is not under project root {}",
                    abs_path.display(),
                    root.display()
                )
            })?;

        let vpath = VirtualPath::new(rel_path);
        let main_id = FileId::new(None, vpath);

        Ok(Self {
            root,
            main_id,
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(FontBook::new()),
            sources: Mutex::new(HashMap::new()),
        })
    }

    /// Resolve a `FileId` to an absolute filesystem path.
    fn resolve_path(&self, id: FileId) -> FileResult<PathBuf> {
        id.vpath()
            .resolve(&self.root)
            .ok_or_else(|| FileError::AccessDenied)
    }

    /// Read and cache a source file.
    fn load_source(&self, id: FileId) -> FileResult<Source> {
        let path = self.resolve_path(id)?;
        let text = std::fs::read_to_string(&path).map_err(|e| FileError::from_io(e, &path))?;
        Ok(Source::new(id, text))
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
        {
            let cache = self.sources.lock().unwrap();
            if let Some(source) = cache.get(&id) {
                return Ok(source.clone());
            }
        }

        let source = self.load_source(id)?;

        let mut cache = self.sources.lock().unwrap();
        cache.insert(id, source.clone());
        Ok(source)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = self.resolve_path(id)?;
        let data = std::fs::read(&path).map_err(|e| FileError::from_io(e, &path))?;
        Ok(Bytes::new(data))
    }

    fn font(&self, _index: usize) -> Option<Font> {
        None
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        let now = chrono_free_today();
        Datetime::from_ymd(now.0, now.1, now.2)
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
fn find_project_root(start_dir: &Path) -> PathBuf {
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

/// Get today's date as (year, month, day) using only the standard library.
///
/// This avoids pulling in `chrono` or `time` as a direct dependency.
/// We compute the date from the Unix timestamp using a well-known algorithm.
fn chrono_free_today() -> (i32, u8, u8) {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Days since Unix epoch (1970-01-01).
    let days = (secs / 86400) as i64;

    // Civil date from day count.
    // Algorithm from Howard Hinnant:
    // https://howardhinnant.github.io/date_algorithms.html#civil_from_days
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64; // day of era [0, 146096]
    let yoe =
        (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    let y = if m <= 2 { y + 1 } else { y };

    (y as i32, m as u8, d as u8)
}
