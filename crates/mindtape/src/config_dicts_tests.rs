#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::path::Path;

use super::{
    BUILTIN_PREFIX, Dict, DictCache, DictCase, DictSpec, Dicts, builtin_dict_names, compile_dicts,
};
use crate::config::parse_config;

/// Compile the `[dicts]` section of a config text against `root`.
fn compile(text: &str, root: &Path) -> Result<Dicts, String> {
    let config = parse_config(text).expect("config parses");
    compile_dicts(&config.dicts, root)
}

/// Compile against a root no dict reads (every source is inline).
fn compile_inline(text: &str) -> Result<Dicts, String> {
    compile(text, Path::new("/nonexistent-root"))
}

fn entries(dicts: &Dicts, name: &str) -> Vec<String> {
    dicts
        .get(name)
        .unwrap_or_else(|| panic!("dict `{name}` compiled"))
        .entries()
        .map(ToOwned::to_owned)
        .collect()
}

#[test]
fn inline_values_are_the_whole_set() {
    let dicts = compile_inline("[dicts.verbs]\nvalues = [\"add\", \"wire\"]\n").unwrap();
    assert_eq!(entries(&dicts, "verbs"), ["add", "wire"]);
}

/// `values`, `path` and `include` all feed one set — the union, de-duplicated.
#[test]
fn values_path_and_include_union_into_one_set() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("dicts")).unwrap();
    std::fs::write(
        dir.path().join("dicts/extra.txt"),
        "# a comment\n\nfix\nwire\n",
    )
    .unwrap();
    let dicts = compile(
        "[dicts.base]\nvalues = [\"ship\"]\n\n\
         [dicts.verbs]\nvalues = [\"add\", \"wire\"]\n\
         path = \"dicts/extra.txt\"\ninclude = [\"base\"]\n",
        dir.path(),
    )
    .unwrap();
    assert_eq!(entries(&dicts, "verbs"), ["add", "fix", "ship", "wire"]);
}

/// A `#` comment and a blank line are skipped; the header the built-in data
/// files carry is exactly this shape.
#[test]
fn a_path_skips_comments_and_blank_lines() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("words.txt"),
        "# header\n#\n\n  add  \n\nfix\n",
    )
    .unwrap();
    let dicts = compile("[dicts.verbs]\npath = \"words.txt\"\n", dir.path()).unwrap();
    assert_eq!(entries(&dicts, "verbs"), ["add", "fix"]);
}

/// `path` may leave the project root, same rule as `[[typst.local-packages]]`.
#[test]
fn a_path_may_leave_the_project_root() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("project")).unwrap();
    std::fs::write(dir.path().join("outside.txt"), "add\n").unwrap();
    let dicts = compile(
        "[dicts.verbs]\npath = \"../outside.txt\"\n",
        &dir.path().join("project"),
    )
    .unwrap();
    assert_eq!(entries(&dicts, "verbs"), ["add"]);
}

#[test]
fn fold_is_the_default_and_matches_a_capitalised_lookup() {
    let dicts = compile_inline("[dicts.verbs]\nvalues = [\"add\"]\n").unwrap();
    let dict = dicts.get("verbs").unwrap();
    assert_eq!(dict.case(), DictCase::Fold);
    assert!(dict.contains("Add"), "fold matches a capitalised lookup");
    assert!(dict.contains("ADD"));
    assert!(!dict.contains("adder"));
}

#[test]
fn exact_compares_bytes() {
    let dicts = compile_inline("[dicts.verbs]\nvalues = [\"add\"]\ncase = \"exact\"\n").unwrap();
    let dict = dicts.get("verbs").unwrap();
    assert_eq!(dict.case(), DictCase::Exact);
    assert!(dict.contains("add"));
    assert!(!dict.contains("Add"), "exact refuses a capitalised lookup");
}

/// A `fold` dict folds an entry declared with capitals, too — the policy is the
/// dict's, not the lookup's.
#[test]
fn fold_normalizes_the_entries_as_well_as_the_lookup() {
    let dicts = compile_inline("[dicts.verbs]\nvalues = [\"Add\"]\n").unwrap();
    assert!(dicts.get("verbs").unwrap().contains("add"));
}

/// `include` unions the target's raw entries in; the including dict's own
/// `case` decides how they are compared.
#[test]
fn an_included_dict_is_read_under_the_includer_s_case() {
    let dicts = compile_inline(
        "[dicts.base]\nvalues = [\"Add\"]\ncase = \"exact\"\n\n\
         [dicts.verbs]\ninclude = [\"base\"]\n",
    )
    .unwrap();
    assert!(dicts.get("base").unwrap().contains("Add"));
    assert!(!dicts.get("base").unwrap().contains("add"));
    assert!(dicts.get("verbs").unwrap().contains("add"));
}

#[test]
fn the_two_builtins_are_available_without_being_declared() {
    let dicts = compile_inline("").unwrap();
    assert_eq!(
        builtin_dict_names(),
        ["mindtape-verbs", "mindtape-dev-verbs"]
    );
    assert_eq!(dicts.get("mindtape-verbs").unwrap().len(), 500);
    assert_eq!(dicts.get("mindtape-dev-verbs").unwrap().len(), 316);
    assert!(dicts.contains("mindtape-verbs", "Add"));
    assert!(dicts.contains("mindtape-dev-verbs", "refactor"));
    assert!(!dicts.contains("mindtape-verbs", "refactor"));
}

#[test]
fn the_dev_verbs_carry_the_common_engineering_verbs() {
    let dicts = compile_inline("").unwrap();
    for verb in [
        "compute",
        "filter",
        "page",
        "roll",
        "switch",
        "delete",
        "group",
        "update",
        "rate-limit",
        "speed",
    ] {
        assert!(dicts.contains("mindtape-dev-verbs", verb), "{verb}");
    }
}

#[test]
fn a_project_dict_may_include_a_builtin() {
    let dicts =
        compile_inline("[dicts.verbs]\nvalues = [\"mt\"]\ninclude = [\"mindtape-verbs\"]\n")
            .unwrap();
    let dict = dicts.get("verbs").unwrap();
    assert_eq!(dict.len(), 501);
    assert!(dict.contains("Add"));
    assert!(dict.contains("mt"));
}

#[test]
fn the_reserved_prefix_is_refused() {
    let err = compile_inline("[dicts.mindtape-mine]\nvalues = [\"add\"]\n").unwrap_err();
    assert!(err.contains("dicts.mindtape-mine"), "{err}");
    assert!(err.contains(BUILTIN_PREFIX), "{err}");
    assert!(err.contains("reserved"), "{err}");
}

#[test]
fn an_unknown_include_suggests_the_nearest_name() {
    let err = compile_inline(
        "[dicts.verbs]\nvalues = [\"add\"]\n\n[dicts.nouns]\ninclude = [\"verb\"]\n",
    )
    .unwrap_err();
    assert!(err.contains("dicts.nouns"), "{err}");
    assert!(err.contains("unknown include `verb`"), "{err}");
    assert!(err.contains("did you mean: verbs?"), "{err}");
}

#[test]
fn an_include_cycle_is_refused() {
    let err = compile_inline(
        "[dicts.a]\nvalues = [\"x\"]\ninclude = [\"b\"]\n\n\
         [dicts.b]\nvalues = [\"y\"]\ninclude = [\"a\"]\n",
    )
    .unwrap_err();
    assert!(err.contains("include cycle"), "{err}");
    assert!(err.contains('a') && err.contains('b'), "{err}");
}

#[test]
fn a_path_that_does_not_resolve_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let err = compile("[dicts.verbs]\npath = \"missing.txt\"\n", dir.path()).unwrap_err();
    assert!(err.contains("dicts.verbs.path"), "{err}");
    assert!(err.contains("missing.txt"), "{err}");
}

#[test]
fn a_dict_with_no_source_is_refused() {
    let err = compile_inline("[dicts.verbs]\ncase = \"exact\"\n").unwrap_err();
    assert!(err.contains("dicts.verbs"), "{err}");
    assert!(err.contains("values"), "{err}");
    assert!(err.contains("path"), "{err}");
    assert!(err.contains("include"), "{err}");
}

/// Distinct from "no source": the dict names a source, and the source is empty.
#[test]
fn a_dict_that_resolves_empty_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("words.txt"), "# only a comment\n").unwrap();
    let err = compile("[dicts.verbs]\npath = \"words.txt\"\n", dir.path()).unwrap_err();
    assert!(err.contains("dicts.verbs"), "{err}");
    assert!(err.contains("no entries"), "{err}");
}

/// Every problem is reported at once, so one config error covers the section.
#[test]
fn every_problem_in_the_section_is_reported_together() {
    let err = compile_inline(
        "[dicts.mindtape-mine]\nvalues = [\"x\"]\n\n[dicts.other]\ncase = \"fold\"\n",
    )
    .unwrap_err();
    assert!(err.contains("dicts.mindtape-mine"), "{err}");
    assert!(err.contains("dicts.other"), "{err}");
}

#[test]
fn an_unknown_dict_name_looks_up_as_a_miss() {
    let dicts = compile_inline("").unwrap();
    assert!(dicts.get("nope").is_none());
    assert!(!dicts.contains("nope", "add"));
}

/// A `Config` answers `in_dict()` without ever having seen a project root: the
/// cache defaults to the built-ins, so a bare `Config::default()` resolves
/// `mindtape-verbs`.
#[test]
fn the_cache_defaults_to_the_builtins_alone() {
    let cache = DictCache::default();
    assert_eq!(
        cache.dicts().names().collect::<Vec<_>>(),
        ["mindtape-dev-verbs", "mindtape-verbs"]
    );
    assert!(
        crate::config::Config::default()
            .compiled_dicts()
            .contains("mindtape-verbs", "Add")
    );
}

/// `parse_config` has no root, so it compiles the section rootless: an inline
/// dict resolves, and a broken section leaves the built-ins alone rather than
/// failing a function documented to check shape only.
#[test]
fn parse_config_hydrates_the_cache_without_a_root() {
    let config = parse_config("[dicts.verbs]\nvalues = [\"add\"]\n").unwrap();
    assert!(config.compiled_dicts().contains("verbs", "Add"));

    let broken = parse_config("[dicts.mindtape-mine]\nvalues = [\"add\"]\n").unwrap();
    assert!(broken.compiled_dicts().get("mindtape-mine").is_none());
    assert!(broken.compiled_dicts().contains("mindtape-verbs", "add"));
}

/// The cache is derived from the `[dicts]` section and the project root, and
/// `Config` carries no root at all — so two configs with identical declared
/// settings stay equal however each one's dictionaries were compiled. Pins the
/// hand-written `PartialEq` on [`DictCache`].
#[test]
fn the_cache_is_not_part_of_config_equality() {
    let mut one = parse_config("[dicts.verbs]\nvalues = [\"add\"]\n").unwrap();
    let other = one.clone();
    one.dict_cache = DictCache::default();

    assert!(one.compiled_dicts().get("verbs").is_none());
    assert!(other.compiled_dicts().get("verbs").is_some());
    assert_eq!(one, other);
}

#[test]
fn an_empty_section_compiles_to_the_builtins_alone() {
    let specs: BTreeMap<String, DictSpec> = BTreeMap::new();
    let dicts = compile_dicts(&specs, Path::new("/nonexistent-root")).unwrap();
    assert_eq!(
        dicts.names().collect::<Vec<_>>(),
        ["mindtape-dev-verbs", "mindtape-verbs"]
    );
}

#[test]
fn is_empty_tracks_the_entry_set() {
    let entries = ["add".to_owned()].into_iter().collect();
    assert!(!Dict::new(entries, DictCase::Fold).is_empty());
    assert!(Dict::default().is_empty());
}
