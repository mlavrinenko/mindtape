//! Working-directory-relative rendering of a task's source path.
//!
//! `Task.file` is stored root-relative (`truth/tasks/x.typ`), but the table's
//! `file` column shows a path relative to the process's working directory —
//! the form an IDE-integrated terminal (Zed, VS Code, `JetBrains`) turns into
//! a ctrl-click with no escape sequences involved
//! (`truth/tasks/opt-in-file-column-for-mt-ls.typ`). [`relativize`] is the
//! pure core; the root comes from [`super::RenderCtx`], so one process can
//! render several projects, and the working directory is read once ([`cwd`]).
//!
//! Only the two surfaces a human reads from a shell come through here: the
//! table cell and `--format path`/`path0`, which exist to be pasted back into
//! a command from the cwd that produced them. The record formats do not —
//! `--format json` and `--format csv` both carry `task.file` unchanged, so a
//! stored record means the same thing whatever directory produced it
//! (`truth/tasks/csv-and-json-disagree-about-what-the-file-field-means.typ`).

use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

/// The process working directory, read once. `None` when the OS will not say,
/// which leaves paths root-relative rather than failing a render over it.
///
/// The directory belongs to the process, not to a project, so it stays a cell
/// here while each project's root rides in [`super::RenderCtx`].
#[must_use]
pub fn cwd() -> Option<&'static Path> {
    static CWD: OnceLock<Option<PathBuf>> = OnceLock::new();
    CWD.get_or_init(|| std::env::current_dir().ok()).as_deref()
}

/// The `file` column's cell text for a task's root-relative source path
/// (`task.file`): relative to the working directory, given the project `root`
/// the path is relative to. An empty `root` (no project) or an unreadable
/// working directory leaves `file` unchanged.
#[must_use]
pub fn cell_text(root: &Path, file: &Path) -> String {
    match cwd() {
        Some(cwd) if !root.as_os_str().is_empty() => {
            relativize(root, cwd, file).display().to_string()
        }
        _ => file.display().to_string(),
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

/// Default `[ls] hyperlink-target` template: the `file://` scheme over the
/// absolute path. Anyone whose editor registers its own URI scheme overrides
/// this in config instead.
pub const DEFAULT_HYPERLINK_TARGET: &str = "file://{abs}";

/// Build the `file` column's hyperlink target URI for `file` (a task's
/// root-relative source path) by substituting `{abs}` and `{rel}` into
/// `template` — the absolute path (`root.join(file)`) and the same
/// working-directory-relative path [`cell_text`] renders.
///
/// Falls back to `file` unchanged for both tokens when `root` is empty or the
/// working directory is unreadable — the same fallback [`cell_text`] uses.
#[must_use]
pub fn hyperlink_target(root: &Path, file: &Path, template: &str) -> String {
    match cwd() {
        Some(cwd) if !root.as_os_str().is_empty() => {
            build_hyperlink_target(root, cwd, file, template)
        }
        _ => template
            .replace("{abs}", &file.display().to_string())
            .replace("{rel}", &file.display().to_string()),
    }
}

/// Pure core of [`hyperlink_target`], with `root`/`cwd` passed explicitly so
/// the substitution is unit-testable without the process working directory —
/// mirrors [`relativize`]'s split from [`cell_text`].
fn build_hyperlink_target(root: &Path, cwd: &Path, file: &Path, template: &str) -> String {
    let abs = root.join(file);
    let rel = relativize(root, cwd, file);
    template
        .replace("{abs}", &abs.display().to_string())
        .replace("{rel}", &rel.display().to_string())
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
    fn cell_text_without_a_root_is_the_root_relative_path() {
        let file = Path::new("truth/tasks/x.typ");
        assert_eq!(cell_text(Path::new(""), file), "truth/tasks/x.typ");
    }

    #[test]
    fn cell_text_reads_the_root_it_is_given() {
        let file = Path::new("x.typ");
        let cwd = cwd().unwrap();
        for root in ["/one/proj", "/two/proj"] {
            let expected = relativize(Path::new(root), cwd, file).display().to_string();
            assert_eq!(cell_text(Path::new(root), file), expected);
        }
    }

    #[test]
    fn build_hyperlink_target_substitutes_abs() {
        let root = Path::new("/home/user/proj");
        let cwd = Path::new("/home/user/proj");
        let file = Path::new("truth/tasks/x.typ");
        let uri = build_hyperlink_target(root, cwd, file, "file://{abs}");
        assert_eq!(uri, "file:///home/user/proj/truth/tasks/x.typ");
    }

    #[test]
    fn build_hyperlink_target_substitutes_rel_from_a_subdirectory() {
        let root = Path::new("/home/user/proj");
        let cwd = Path::new("/home/user/proj/sub");
        let file = Path::new("truth/tasks/x.typ");
        let uri = build_hyperlink_target(root, cwd, file, "editor://{rel}");
        assert_eq!(uri, "editor://../truth/tasks/x.typ");
    }

    #[test]
    fn build_hyperlink_target_supports_both_tokens_in_one_template() {
        let root = Path::new("/proj");
        let cwd = Path::new("/proj");
        let file = Path::new("x.typ");
        let uri = build_hyperlink_target(root, cwd, file, "{rel} -> {abs}");
        assert_eq!(uri, "x.typ -> /proj/x.typ");
    }

    #[test]
    fn hyperlink_target_joins_the_root_it_is_given() {
        let uri = hyperlink_target(Path::new("/one/proj"), Path::new("x.typ"), "{abs}");
        assert_eq!(uri, "/one/proj/x.typ");
    }

    #[test]
    fn hyperlink_target_without_a_root_is_the_file_unchanged() {
        let file = Path::new("truth/tasks/x.typ");
        assert_eq!(
            hyperlink_target(Path::new(""), file, "file://{abs}"),
            "file://truth/tasks/x.typ"
        );
    }
}
