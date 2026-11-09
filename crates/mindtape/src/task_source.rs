//! The one policy naming which project paths hold task sources.
//!
//! A task source is a regular `.typ` file under a configured
//! `[[tasks.roots]]` entry, outside any path component named `.mindtape` or
//! `.git` (case-insensitive) and outside every `[templates] dirs` entry. The
//! loader, and the commit, write-guard and `mt fix` layers after it, ask
//! [`TaskSource`] instead of re-deriving that rule.
//!
//! Matching is lexical: a path is normalized by resolving `.` and `..`
//! without touching the filesystem, then compared as project-relative and
//! absolute forms. A symlink is never a source, even when it points at a
//! regular `.typ` file.

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use crate::config::Config;
use crate::config::roots::normalize_path;
use crate::loader::LoadWarning;
use crate::model::task_spans::TaskCallSpans;

/// The message a reserved `.typ` file's warning carries.
const RESERVED_MESSAGE: &str = "`task.with` call in a reserved path";

/// The hint attached to the reserved-path warning.
const RESERVED_HINT: &str = "`.mindtape`, `.git` and `[templates] dirs` hold no task sources";

/// What a [`TaskPath`] names on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskPathKind {
    /// A regular file. Only this kind can be a task source.
    Regular,
    /// A symbolic link, even one pointing at a regular `.typ` file.
    Symlink,
    /// Nothing at the path, or an entry that is neither a file nor a symlink,
    /// such as a directory.
    Missing,
}

/// One path as the task-source policy sees it: its project-relative and
/// absolute forms, and the kind of entry at the absolute form.
///
/// Both forms are lexical. A relative input resolves against the project root,
/// never the working directory; `..` below the root stays in the relative
/// form. A path on another filesystem root (a Windows drive, say) has no
/// relative form and keeps its absolute one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskPath {
    rel: PathBuf,
    abs: PathBuf,
    kind: TaskPathKind,
}

impl TaskPath {
    /// Classify `path` against `project_root`.
    ///
    /// `path` may be project-relative or absolute. `.` is dropped and `..` is
    /// resolved without touching the filesystem. The kind is read from the
    /// absolute form: a regular file, a symlink, or anything else (absent, a
    /// directory) is [`TaskPathKind::Missing`].
    #[must_use]
    pub fn new(project_root: &Path, path: &Path) -> Self {
        let root = absolute(project_root);
        Self::against(&root, path)
    }

    /// [`Self::new`] against an already normalized project root.
    fn against(root: &Path, path: &Path) -> Self {
        let (rel, abs) = if path.is_absolute() {
            let abs = normalize_path(path);
            (relative(root, &abs), abs)
        } else {
            let rel = normalize_path(path);
            let abs = normalize_path(&root.join(&rel));
            (rel, abs)
        };
        let kind = kind_of(&abs);
        Self { rel, abs, kind }
    }

    /// The path relative to the project root.
    #[must_use]
    pub fn rel(&self) -> &Path {
        &self.rel
    }

    /// The path made absolute and lexical-normalized.
    #[must_use]
    pub fn abs(&self) -> &Path {
        &self.abs
    }

    /// The kind of entry at [`Self::abs`].
    #[must_use]
    pub const fn kind(&self) -> TaskPathKind {
        self.kind
    }
}

/// The task-source policy of one project, derived from its [`Config`].
///
/// The configured roots and template directories are held in separate,
/// normalized absolute forms; [`crate::config::RootSpec::dir`] keeps its
/// string match-key semantics for commit and tag derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSource {
    /// The project root the relative forms resolve against, normalized.
    root: PathBuf,
    /// Every `[[tasks.roots]]` entry, normalized and absolute.
    roots: Vec<PathBuf>,
    /// Every `[templates] dirs` entry, normalized and absolute.
    templates: Vec<PathBuf>,
}

impl TaskSource {
    /// Build the policy for the project at `root` from `config`.
    ///
    /// No path is read here: only the config's roots and template dirs are
    /// normalized. [`Self::is_source`] then reads the one path it classifies.
    #[must_use]
    pub fn new(root: &Path, config: &Config) -> Self {
        let root = absolute(root);
        let roots = (config.tasks.roots.iter())
            .map(|spec| anchored(&root, &spec.normalized()))
            .collect();
        let templates = (config.templates.normalized_dirs().iter())
            .map(|dir| anchored(&root, dir))
            .collect();
        Self {
            root,
            roots,
            templates,
        }
    }

    /// The normalized project root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The configured task roots, normalized and absolute.
    #[must_use]
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// The configured template directories, normalized and absolute.
    #[must_use]
    pub fn templates(&self) -> &[PathBuf] {
        &self.templates
    }

    /// Classify `path`, project-relative or absolute, against this policy's
    /// project root.
    #[must_use]
    pub fn task_path(&self, path: &Path) -> TaskPath {
        TaskPath::against(&self.root, path)
    }

    /// Whether `path` is a task source: a regular `.typ` file under a
    /// configured root, outside every reserved component and template dir.
    #[must_use]
    pub fn is_source(&self, path: &TaskPath) -> bool {
        path.kind == TaskPathKind::Regular && self.would_be_source(path)
    }

    /// Whether a task filed at `path` would be a source, ignoring the kind of
    /// entry there now: a `.typ` file under a configured root, outside every
    /// reserved component and template dir.
    ///
    /// This is the lexical half of [`Self::is_source`], and the question a
    /// destination probe asks about a file that does not exist yet.
    #[must_use]
    pub fn would_be_source(&self, path: &TaskPath) -> bool {
        is_typ(&path.abs)
            && self.under_root(path)
            && !has_reserved_component(&path.rel)
            && !self.under_template(path)
    }

    /// Whether `path` is a regular `.typ` file that would sit under a
    /// configured root, but is excluded by a reserved `.mindtape`/`.git`
    /// component or by a configured template dir.
    ///
    /// The loader uses this to explain a task call it deliberately does not
    /// load; a file outside the roots is not reserved, it is simply elsewhere.
    #[must_use]
    pub fn is_reserved(&self, path: &TaskPath) -> bool {
        path.kind == TaskPathKind::Regular
            && is_typ(&path.abs)
            && self.under_root(path)
            && (has_reserved_component(&path.rel) || self.under_template(path))
    }

    /// The one warning a reserved `.typ` file earns when `source` holds a
    /// `task.with(…)` call: a task call at a path the loader does not load.
    /// `None` when `path` is not reserved or has no such call.
    #[must_use]
    pub fn reserved_warning(&self, path: &TaskPath, source: &str) -> Option<LoadWarning> {
        if !self.is_reserved(path) {
            return None;
        }
        let line = TaskCallSpans::parse(source).call_line()?;
        Some(LoadWarning {
            file: path.rel.clone(),
            line: Some(line),
            column: None,
            message: RESERVED_MESSAGE.to_owned(),
            hints: vec![RESERVED_HINT.to_owned()],
        })
    }

    /// [`Self::reserved_warning`] read from the file at `path`. `None` when it
    /// is not reserved, cannot be read, or holds no task call.
    pub(crate) fn reserved_warning_for_file(&self, path: &TaskPath) -> Option<LoadWarning> {
        if !self.is_reserved(path) {
            return None;
        }
        let source = std::fs::read_to_string(&path.abs).ok()?;
        self.reserved_warning(path, &source)
    }

    /// Every `.typ` file in `dir` the policy admits or reserves, skipping a
    /// directory the walk has no reason to enter and never a symlink.
    pub(crate) fn typ_files(&self, dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        self.collect_typ_files(dir, &mut files);
        files
    }

    fn collect_typ_files(&self, dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                if self.descends_into(&path) {
                    self.collect_typ_files(&path, out);
                }
            } else if is_typ(&path) {
                out.push(path);
            }
        }
    }

    /// Whether a directory walk should descend into `dir`.
    ///
    /// A reserved directory is skipped, except one on the way to a configured
    /// template dir or inside one: those files are read only to warn about a
    /// task call, never to load a task.
    pub(crate) fn descends_into(&self, dir: &Path) -> bool {
        let abs = self.absolute(dir);
        let template_side = (self.templates.iter())
            .any(|template| template.starts_with(&abs) || abs.starts_with(template));
        template_side || !has_reserved_component(&relative(&self.root, &abs))
    }

    /// `path`, project-relative or absolute, as a normalized absolute path.
    fn absolute(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            normalize_path(path)
        } else {
            normalize_path(&self.root.join(path))
        }
    }

    fn under_root(&self, path: &TaskPath) -> bool {
        self.roots.iter().any(|root| path.abs.starts_with(root))
    }

    fn under_template(&self, path: &TaskPath) -> bool {
        self.templates
            .iter()
            .any(|template| path.abs.starts_with(template))
    }
}

/// `path` made absolute when possible, then lexical-normalized.
fn absolute(path: &Path) -> PathBuf {
    let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    normalize_path(&path)
}

/// `dir` resolved against `root` unless it is already absolute, then
/// lexical-normalized.
fn anchored(root: &Path, dir: &Path) -> PathBuf {
    if dir.is_absolute() {
        normalize_path(dir)
    } else {
        normalize_path(&root.join(dir))
    }
}

/// `path` relative to the normalized absolute `root`, lexically.
fn relative(root: &Path, path: &Path) -> PathBuf {
    if let Ok(rel) = path.strip_prefix(root) {
        return rel.to_path_buf();
    }
    let root_parts: Vec<Component<'_>> = root.components().collect();
    let path_parts: Vec<Component<'_>> = path.components().collect();
    let common = (root_parts.iter())
        .zip(&path_parts)
        .take_while(|(left, right)| left == right)
        .count();
    if common == 0 {
        return path.to_path_buf();
    }
    let mut rel = PathBuf::new();
    for _ in common..root_parts.len() {
        rel.push("..");
    }
    for part in path_parts.iter().skip(common) {
        rel.push(part.as_os_str());
    }
    rel
}

/// The kind of entry at `path`, without following a final symlink.
fn kind_of(path: &Path) -> TaskPathKind {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => TaskPathKind::Symlink,
        Ok(metadata) if metadata.is_file() => TaskPathKind::Regular,
        _ => TaskPathKind::Missing,
    }
}

/// Whether `path` ends in the `.typ` extension, case-sensitively.
fn is_typ(path: &Path) -> bool {
    path.extension().and_then(OsStr::to_str) == Some("typ")
}

/// Whether any component of `path` is the tool's `.mindtape` or `.git`
/// metadata directory, in any ASCII case.
pub(crate) fn has_reserved_component(path: &Path) -> bool {
    (path.components()).any(|component| match component {
        Component::Normal(name) => {
            let name = name.to_string_lossy();
            name.eq_ignore_ascii_case(".mindtape") || name.eq_ignore_ascii_case(".git")
        }
        _ => false,
    })
}

#[cfg(test)]
#[path = "task_source_tests.rs"]
mod tests;
