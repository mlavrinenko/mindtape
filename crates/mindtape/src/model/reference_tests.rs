#![allow(clippy::unwrap_used)]

use crate::test_support::tempdir;
use std::path::{Path, PathBuf};

use super::*;
use crate::test_support::bare_task;

fn tasks(stems: &[&str]) -> Vec<Task> {
    stems.iter().map(|stem| bare_task(stem)).collect()
}

/// The task the linker in [`target`]'s tests lives in.
const LINKER: &str = "tasks/docs/alpha.typ";

/// A task at each root-relative file, named by its file stem.
fn tasks_at(files: &[&str]) -> Vec<Task> {
    files
        .iter()
        .map(|file| {
            let mut task = bare_task(&Path::new(file).file_stem().unwrap().to_string_lossy());
            task.file = (*file).into();
            task
        })
        .collect()
}

fn hashes_of(tasks: &[Task]) -> StemHashes {
    StemHashes::new(tasks.iter().map(|task| task.id.as_str()))
}

/// `reference` typed on the command line, from the root.
fn resolve(tasks: &[Task], root: &Path, reference: &str) -> Result<(String, PathBuf), RefError> {
    resolve_ref(tasks, root, Base::Cwd(root), &hashes_of(tasks), reference)
        .map(|got| (got.task.id.clone(), got.path))
}

/// The literal a `links:` target `reference` in [`LINKER`] writes.
fn target(tasks: &[Task], reference: &str) -> Result<PathBuf, RefError> {
    let base = Base::Linker(Path::new(LINKER));
    let root = Path::new("/p");
    resolve_ref(tasks, root, base, &hashes_of(tasks), reference)
        .map(|got| base.path_to(root, &got.task.file))
}

/// The two names besides a path that a target gives `gamma`: its stem and its
/// stem hash.
fn names_of_gamma(tasks: &[Task]) -> [String; 2] {
    let hash = hashes_of(tasks).hash_of("gamma").unwrap().to_owned();
    ["gamma".to_owned(), hash]
}

fn root_with(stems: &[&str]) -> tempfile::TempDir {
    let dir = tempdir();
    std::fs::create_dir_all(dir.path().join("tasks")).unwrap();
    for stem in stems {
        std::fs::write(dir.path().join(format!("tasks/{stem}.typ")), "").unwrap();
    }
    dir
}

#[test]
fn a_stem_resolves_to_its_task_and_file() {
    let tasks = tasks(&["alpha", "beta"]);
    let got = resolve(&tasks, Path::new("/p"), "beta").unwrap();
    assert_eq!(
        got,
        (
            "beta".to_owned(),
            Path::new("/p/tasks/beta.typ").to_path_buf()
        )
    );
}

#[test]
fn a_stem_hash_resolves_when_no_stem_matches() {
    let tasks = tasks(&["alpha", "beta"]);
    let hash = StemHashes::new(["alpha", "beta"])
        .hash_of("beta")
        .unwrap()
        .to_uppercase();
    assert_eq!(resolve(&tasks, Path::new("/p"), &hash).unwrap().0, "beta");
}

#[test]
fn an_exact_stem_wins_over_a_hash_it_spells() {
    let spelled = StemHashes::new(["alpha"])
        .hash_of("alpha")
        .unwrap()
        .to_owned();
    let tasks = tasks(&["alpha", &spelled]);
    assert_eq!(
        resolve(&tasks, Path::new("/p"), &spelled).unwrap().0,
        spelled
    );
}

#[test]
fn a_resolution_says_how_the_reference_named_its_task() {
    let dir = root_with(&["alpha"]);
    let tasks = tasks(&["alpha"]);
    let hashes = hashes_of(&tasks);
    let hash = hashes.hash_of("alpha").unwrap().to_owned();
    let base = Base::Cwd(dir.path());
    for (reference, via) in [
        ("alpha", Via::Stem),
        (hash.as_str(), Via::Hash),
        ("tasks/alpha.typ", Via::Path),
    ] {
        let got = resolve_ref(&tasks, dir.path(), base, &hashes, reference).unwrap();
        assert_eq!(got.via, via, "{reference}");
    }
}

#[test]
fn a_reference_naming_nothing_says_why() {
    let tasks = tasks(&["alpha"]);
    let refusal = |reference| resolve(&tasks, Path::new("/p"), reference).unwrap_err();
    assert!(matches!(refusal(""), RefError::Empty));
    assert!(matches!(
        refusal("alpah"),
        RefError::NoMatch { reference, nearest: Some(near) } if reference == "alpah" && near == "alpha"
    ));
    assert!(matches!(
        refusal("zzzz"),
        RefError::NoMatch { nearest: None, .. }
    ));
}

#[test]
fn a_shared_hash_is_ambiguous_naming_each_stem_and_its_hash() {
    let tasks = tasks(&["a-1", "a-2"]);
    let hashes = StemHashes::with_hasher(["a-1", "a-2"], |stem| {
        u64::from(stem.as_bytes().last().copied().unwrap_or(0))
    });
    let prefix = hashes.hash_of("a-1").unwrap().get(..3).unwrap().to_owned();
    let root = Path::new("/p");
    let got = resolve_ref(&tasks, root, Base::Cwd(root), &hashes, &prefix);
    let want: Vec<(String, String)> = ["a-1", "a-2"]
        .iter()
        .map(|stem| (hashes.hash_of(stem).unwrap().to_owned(), (*stem).to_owned()))
        .collect();
    assert!(
        matches!(got, Err(RefError::Ambiguous { ref candidates, .. }) if candidates == &want),
        "{got:?}"
    );
}

#[test]
fn a_path_resolves_from_the_working_directory() {
    let dir = root_with(&["alpha"]);
    let tasks = tasks(&["alpha"]);
    let abs = dir.path().join("tasks/alpha.typ").canonicalize().unwrap();
    for reference in ["tasks/alpha.typ", abs.to_str().unwrap()] {
        let got = resolve(&tasks, dir.path(), reference).unwrap();
        assert_eq!(got, ("alpha".to_owned(), abs.clone()), "{reference}");
    }
}

#[test]
fn a_backslash_reads_as_a_path_not_a_stem_hash() {
    let dir = root_with(&["alpha"]);
    let got = resolve(&tasks(&["alpha"]), dir.path(), "tasks\\alpha");
    assert!(matches!(got, Err(RefError::Path { .. })), "{got:?}");
}

#[test]
fn a_path_naming_no_task_says_where() {
    let dir = root_with(&["alpha"]);
    std::fs::write(dir.path().join("tasks/stray.typ"), "").unwrap();
    let got = resolve(&tasks(&["alpha"]), dir.path(), "tasks/stray.typ");
    assert!(matches!(got, Err(RefError::NoTaskAt(_))), "{got:?}");
}

#[test]
fn a_missing_path_is_a_path_error() {
    let dir = root_with(&[]);
    let got = resolve(&tasks(&["alpha"]), dir.path(), "a/b");
    assert!(matches!(got, Err(RefError::Path { .. })), "{got:?}");
    assert!(got.unwrap_err().to_string().starts_with("resolve "));
}

#[test]
fn only_a_path_shaped_reference_is_a_path() {
    for path in ["a/b", "a\\b", "x.typ", "/abs"] {
        assert!(is_path_ref(path), "{path}");
    }
    assert!(!is_path_ref("alpha"));
}

// --- a `links:` target, read from the linking file ---------------------------

#[test]
fn a_target_path_reads_from_the_linking_file_without_the_disk() {
    let tasks = tasks_at(&[LINKER, "tasks/cli/gamma.typ"]);
    let got = target(&tasks, "./../cli/gamma.typ").unwrap();
    assert_eq!(got, Path::new("../cli/gamma.typ"));
}

#[test]
fn a_target_path_naming_no_task_says_where_it_reads() {
    let tasks = tasks_at(&[LINKER]);
    for (reference, at) in [
        ("gamma.typ", "tasks/docs/gamma.typ"),
        ("../../../out.typ", "../../../out.typ"),
    ] {
        let got = target(&tasks, reference);
        assert!(
            matches!(got, Err(RefError::NoTaskAt(ref path)) if path == Path::new(at)),
            "{got:?}"
        );
    }
}

#[test]
fn a_target_stem_or_hash_in_another_folder_writes_its_path() {
    let tasks = tasks_at(&[LINKER, "tasks/cli/gamma.typ"]);
    let written = names_of_gamma(&tasks).map(|name| target(&tasks, &name).unwrap());
    assert_eq!(
        written,
        ["../cli/gamma.typ", "../cli/gamma.typ"].map(PathBuf::from)
    );
}

#[test]
fn a_shared_stem_names_the_task_beside_the_base() {
    let tasks = tasks_at(&["tasks/web/gamma.typ", "tasks/docs/gamma.typ"]);
    assert_eq!(target(&tasks, "gamma").unwrap(), Path::new("gamma.typ"));

    let root = Path::new("/p");
    let cwd = root.join("tasks/web");
    let got = resolve_ref(&tasks, root, Base::Cwd(&cwd), &hashes_of(&tasks), "gamma").unwrap();
    assert_eq!(got.task.file, Path::new("tasks/web/gamma.typ"));
}

#[test]
fn a_shared_stem_beside_no_base_is_refused_naming_each_path_from_it() {
    let tasks = tasks_at(&["tasks/web/gamma.typ", "tasks/cli/gamma.typ"]);
    let want = ["../cli/gamma.typ", "../web/gamma.typ"].map(PathBuf::from);
    for name in names_of_gamma(&tasks) {
        let got = target(&tasks, &name);
        assert!(
            matches!(got, Err(RefError::SharedStem { ref reference, ref paths })
                if reference == &name && paths == &want),
            "{got:?}"
        );
    }
}

#[test]
fn a_base_outside_the_root_names_each_path_in_full() {
    let base = Base::Cwd(Path::new("/elsewhere"));
    assert_eq!(
        base.path_to(Path::new("/p"), Path::new("tasks/a.typ")),
        Path::new("/p/tasks/a.typ")
    );
}
