//! End-to-end: the `SLUG#` column prints a stem hash, and every `<task>`
//! argument accepts one.
//!
//! The hash comes from the filename stems of every task the project loads, so
//! a filtered listing prints the same hash as the full one, and a flip or a new
//! task leaves it alone unless the new stem shares its prefix.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::collections::BTreeMap;
use std::path::Path;

use mindtape::StemHashes;

use crate::support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n";

/// A `proposed` task titled `Task <stem>` carrying `tags`.
fn write_task(dir: &Path, stem: &str, tags: &[&str]) {
    let tags: String = tags.iter().map(|tag| format!("\"{tag}\",")).collect();
    std::fs::write(
        dir.join("tasks").join(format!("{stem}.typ")),
        format!(
            "#import \"@local/mindtape:0.2.0\": *\n\n\
             #show: task.with(\n  title: \"Task {stem}\",\n  \
             status: proposed(2026, 8, 1),\n  priority: 5,\n  tags: ({tags}),\n)\n"
        ),
    )
    .unwrap();
}

fn project(tasks: &[(&str, &[&str])]) -> tempfile::TempDir {
    let dir = support::scaffold_project(CONFIG);
    for (stem, tags) in tasks {
        write_task(dir.path(), stem, tags);
    }
    dir
}

/// The `SLUG#` cell `mt ls <args…>` prints on the row titled `Task <stem>`.
fn listed_hash(dir: &Path, args: &[&str], stem: &str) -> String {
    let listing = support::stdout(dir, &[&["ls"], args].concat());
    assert!(
        listing.starts_with("SLUG#"),
        "the first column is SLUG#: {listing}"
    );
    let title = format!("Task {stem}");
    listing
        .lines()
        .find(|line| line.contains(&title))
        .and_then(|line| line.split_whitespace().next())
        .unwrap_or_else(|| panic!("no {title:?} row in:\n{listing}"))
        .to_owned()
}

/// The stem `mt eval <reference> --json` resolves to.
fn eval_id(dir: &Path, reference: &str) -> String {
    let text = support::stdout(dir, &["eval", reference, "--json"]);
    let task: serde_json::Value = serde_json::from_str(&text).unwrap();
    support::at(&task, "/id").as_str().unwrap().to_owned()
}

/// The three-digit stem hash a stem has on its own.
fn short_code(stem: &str) -> String {
    StemHashes::new([stem]).hash_of(stem).unwrap().to_owned()
}

/// Two stems whose codes share their first three digits, and those digits.
fn colliding_pair() -> (String, String, String) {
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    for n in 0..5000 {
        let stem = format!("task-{n}");
        let code = short_code(&stem);
        if let Some(first) = seen.insert(code.clone(), stem.clone()) {
            return (first, stem, code);
        }
    }
    panic!("no shared prefix among 5000 stems");
}

#[test]
fn a_hash_from_a_filtered_listing_resolves_to_its_row() {
    let dir = project(&[
        ("alpha", &[]),
        ("bravo", &["found"]),
        ("charlie", &[]),
        ("delta", &["found"]),
    ]);
    let hash = listed_hash(dir.path(), &["has_tag(\"found\")"], "delta");

    assert_eq!(eval_id(dir.path(), &hash), "delta");
}

#[test]
fn a_hash_survives_a_flip_and_a_new_task() {
    let dir = project(&[("alpha", &[]), ("bravo", &[]), ("charlie", &[])]);
    let before = listed_hash(dir.path(), &[], "charlie");

    support::run_ok(dir.path(), &["wip", "alpha"]);
    support::run_ok(dir.path(), &["done", "bravo"]);
    // A stem whose code shares no digit prefix with charlie's.
    let added = (0..)
        .map(|n| format!("echo-{n}"))
        .find(|stem| short_code(stem).get(..1) != before.get(..1))
        .unwrap();
    write_task(dir.path(), &added, &[]);

    assert_eq!(listed_hash(dir.path(), &[], "charlie"), before);
    assert_eq!(eval_id(dir.path(), &before), "charlie");
}

#[test]
fn an_exact_stem_wins_over_a_hash_it_spells() {
    let other = "alpha";
    let stem = short_code(other);
    let dir = project(&[(other, &[]), (&stem, &[])]);

    assert_eq!(eval_id(dir.path(), &stem), stem);
}

#[test]
fn an_ambiguous_hash_is_refused_naming_each_candidate() {
    let (first, second, prefix) = colliding_pair();
    let dir = project(&[(&first, &[]), (&second, &[])]);
    let first_hash = listed_hash(dir.path(), &[], &first);
    let second_hash = listed_hash(dir.path(), &[], &second);

    let out = support::mt(dir.path())
        .args(["eval", &prefix])
        .output()
        .unwrap();

    assert!(!out.status.success(), "an ambiguous hash must be refused");
    let err = String::from_utf8_lossy(&out.stderr);
    for expected in [&first, &second, &first_hash, &second_hash] {
        assert!(err.contains(expected.as_str()), "{expected} missing: {err}");
    }
}

/// Stems whose codes start with a filter word, normalised: `task-65480` codes
/// to `hasbbdqps9xcc` (the `has` function) and `task-475620` to `d0ned...`
/// (the `done` status, `o` read as `0`).
const SPELLS_HAS: &str = "task-65480";
const SPELLS_DONE: &str = "task-475620";

/// `mt rm` reads a bare filter word as a filter even when a stem hash spells
/// it, so the word never retires the task whose hash it happens to be.
#[test]
fn rm_reads_a_filter_word_before_a_stem_hash() {
    let dir = project(&[(SPELLS_HAS, &[]), (SPELLS_DONE, &[]), ("alpha", &[])]);
    for (word, stem) in [("has", SPELLS_HAS), ("done", SPELLS_DONE)] {
        assert!(
            word.replace('o', "0").starts_with(&short_code(stem)),
            "fixture: {stem} must spell {word}"
        );
        // Every command but `mt rm` takes the word as that task's stem hash.
        assert_eq!(eval_id(dir.path(), word), stem, "fixture: {word} is a hash");

        let out = support::mt(dir.path())
            .args(["rm", word, "--dry-run", "-y"])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            !stdout.contains(stem),
            "`mt rm {word}` must not plan {stem}: {stdout}"
        );
        assert!(
            !out.status.success(),
            "`{word}` is no boolean filter, so `mt rm {word}` is refused"
        );
    }
}

/// A stem spelling another task's hash would capture it, because an exact stem
/// wins over a hash. The hash lengthens past the stem instead.
#[test]
fn a_hash_never_spells_another_tasks_stem() {
    let spelled = short_code("alpha");
    let dir = project(&[("alpha", &[]), (&spelled, &[])]);

    let hash = listed_hash(dir.path(), &[], "alpha");

    assert_ne!(hash, spelled, "alpha's hash is another task's stem");
    assert!(hash.starts_with(&spelled), "{hash}");
    assert_eq!(eval_id(dir.path(), &hash), "alpha");
    assert_eq!(eval_id(dir.path(), &spelled), spelled);
}

/// A word the filter language reads on its own never prints as a hash, so
/// `mt rm <hash>` cannot be read as that filter.
#[test]
fn a_hash_never_spells_a_filter_word() {
    let dir = project(&[(SPELLS_HAS, &[]), ("alpha", &[])]);

    let hash = listed_hash(dir.path(), &[], SPELLS_HAS);

    assert_ne!(hash, "has");
    assert!(hash.starts_with("has"), "{hash}");
    assert_eq!(eval_id(dir.path(), &hash), SPELLS_HAS);
}
