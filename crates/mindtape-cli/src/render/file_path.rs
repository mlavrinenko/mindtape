//! Working-directory-relative rendering of a task's source path.
//!
//! `Task.file` is stored root-relative (`truth/tasks/x.typ`), but the `file`
//! column shows a path relative to the process's working directory — the
//! form an IDE-integrated terminal (Zed, VS Code, `JetBrains`) turns into a
//! ctrl-click with no escape sequences involved
//! (`truth/tasks/opt-in-file-column-for-mt-ls.typ`). [`relativize`] is the
//! pure core; the process-global `(root, cwd)` pair is recorded once by
//! [`crate::util::open`], mirroring the `WORKTREE` and `COLOR_ENABLED`
//! one-shot cells ([`crate::util`], [`super::color`]).

use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

/// Process-global `(root, cwd)` pair, set once by [`init`]. Unset in unit
/// tests and any caller that never opens a project, which is exactly when
/// [`cell_text`] falls back to the root-relative path unchanged.
static CONTEXT: OnceLock<(PathBuf, PathBuf)> = OnceLock::new();

/// Record the project root and process working directory for this run.
///
/// Must be called at most once with meaningful effect (from
/// [`crate::util::open`]); later calls are ignored, matching the
/// `WORKTREE`/`COLOR_ENABLED` one-shot pattern.
pub fn init(root: &Path, cwd: &Path) {
    let _ = CONTEXT.set((root.to_path_buf(), cwd.to_path_buf()));
}

/// The `file` column's cell text for a task's root-relative source path
/// (`task.file`): relative to the process's working directory when [`init`]
/// has run, else `file` unchanged (root-relative) — what unit tests and any
/// non-`open()` caller see.
#[must_use]
pub fn cell_text(file: &Path) -> String {
    match CONTEXT.get() {
        Some((root, cwd)) => relativize(root, cwd, file).display().to_string(),
        None => file.display().to_string(),
    }
}

/// Express `root.join(file)` relative to `cwd`.
///
/// `file` is already relative to `root`, so only locating `root` relative to
/// `cwd` requires walking a shared prefix; `file`'s own components ride along
/// unchanged. Concretely: drop the components `root` and `cwd` share, emit one
/// `..` per remaining `cwd` component, then descend through `root`'s
/// remaining components, then `file`. Always relative; falls back to the
/// absolute path only when `root` and `cwd` share no common ancestor at all
/// (e.g. distinct Windows drive prefixes).
#[must_use]
pub fn relativize(root: &Path, cwd: &Path, file: &Path) -> PathBuf {
    match ascend(root, cwd) {
        Some(prefix) => prefix.join(file),
        None => root.join(file),
    }
}

/// The relative path from `from` to `to` (both absolute): shared-prefix
/// components dropped, one `..` per leftover `from` component, then `to`'s
/// leftover components appended. `None` when `to` and `from` disagree on
/// their very first component (e.g. distinct Windows drive letters) — the one
/// case with no relative path between them at all.
fn ascend(to: &Path, from: &Path) -> Option<PathBuf> {
    let to_parts: Vec<Component<'_>> = to.components().collect();
    let from_parts: Vec<Component<'_>> = from.components().collect();
    if let (Some(first_to), Some(first_from)) = (to_parts.first(), from_parts.first())
        && first_to != first_from
    {
        return None;
    }
    let shared = to_parts
        .iter()
        .zip(from_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let mut result = PathBuf::new();
    for _ in shared..from_parts.len() {
        result.push("..");
    }
    for part in to_parts.iter().skip(shared) {
        result.push(part.as_os_str());
    }
    Some(result)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn same_directory_is_the_root_relative_path_unchanged() {
        let root = Path::new("/home/user/proj");
        let cwd = Path::new("/home/user/proj");
        let file = Path::new("truth/tasks/x.typ");
        assert_eq!(
            relativize(root, cwd, file),
            PathBuf::from("truth/tasks/x.typ")
        );
    }

    #[test]
    fn cwd_is_root_itself_matches_same_directory() {
        // The "root itself" case the spec calls out is the same computation
        // as same-directory: cwd == root either way.
        let root = Path::new("/a/proj");
        let cwd = Path::new("/a/proj");
        let file = Path::new("x.typ");
        assert_eq!(relativize(root, cwd, file), PathBuf::from("x.typ"));
    }

    #[test]
    fn subdirectory_of_root_ascends_one_level() {
        let root = Path::new("/home/user/proj");
        let cwd = Path::new("/home/user/proj/sub");
        let file = Path::new("truth/tasks/x.typ");
        assert_eq!(
            relativize(root, cwd, file),
            PathBuf::from("../truth/tasks/x.typ")
        );
    }

    #[test]
    fn nested_subdirectory_ascends_multiple_levels() {
        let root = Path::new("/home/user/proj");
        let cwd = Path::new("/home/user/proj/a/b");
        let file = Path::new("x.typ");
        assert_eq!(relativize(root, cwd, file), PathBuf::from("../../x.typ"));
    }

    #[test]
    fn sibling_directory_ascends_then_descends() {
        let root = Path::new("/home/user/proj");
        let cwd = Path::new("/home/user/sibling");
        let file = Path::new("truth/tasks/x.typ");
        assert_eq!(
            relativize(root, cwd, file),
            PathBuf::from("../proj/truth/tasks/x.typ")
        );
    }

    #[test]
    fn cell_text_falls_back_to_root_relative_when_context_unset() {
        // `CONTEXT` is process-global and never set in this test binary
        // (nothing here calls `init`), which is exactly the state unit tests
        // and any non-`open()` caller observe.
        let file = Path::new("truth/tasks/x.typ");
        assert_eq!(cell_text(file), "truth/tasks/x.typ");
    }
}
