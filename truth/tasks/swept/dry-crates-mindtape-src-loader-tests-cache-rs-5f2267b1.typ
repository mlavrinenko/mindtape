#import "@local/mindtape:0.2.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "extract the clone at crates/mindtape/src/loader/tests_cache.rs:55 (5 copies)",
  kind: "refactor",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 9,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  assignee: "agent",
  tags: ("swept", "dry"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a clone the gate tolerates lives in one helper, or carries a
        comment saying why it matches, so the baseline shrinks],
      docs-match-code: neutral,
    ),
  ),
  status: done(2026, 10, 13),
)

== Problem

+ Run `just ask-refactoring`.
+ 🔴 This clone appears 5 times across 1 file, 92 jscpd tokens at most. Extracting it saves about 333 tokens net of the helper and its calls.
  - #raw("crates/mindtape/src/loader/tests_cache.rs:55")
  - #raw("crates/mindtape/src/loader/tests_cache.rs:71")
  - #raw("crates/mindtape/src/loader/tests_cache.rs:104")
  - #raw("crates/mindtape/src/loader/tests_cache.rs:158")
  - #raw("crates/mindtape/src/loader/tests_cache.rs:169")

The first copy reads:

```rust
    doctor_cache_title(&sole_cache_entry(dir.path()), "FROM-CACHE");
    let second = load_project(dir.path(), &Config::default());
    assert_eq!(second.tasks.first().unwrap().title, "FROM-CACHE");
}

#[test]
fn edited_file_misses_and_reevals() {
    let dir = setup();
```

`just check-dry` fails a new clone, and `.jscpd-baseline.json` lists this one as
older than the gate. The arm joins jscpd's pairs into a family by shared
location, so a location may belong to a neighbouring window.

== TO-BE

- 🟢 The clone lives in one helper, or its copies carry a `jscpd:ignore`
  marker whose comment says why they match, and `just dry-baseline` has
  dropped it from the baseline.

== Changes

- `crates/mindtape/src/loader/tests_cache.rs` and every other file the Problem lists: extract a helper and call it from each copy, or mark the copies as intended with `// jscpd:ignore-start` and `// jscpd:ignore-end` and a comment saying why the match is a coincidence. Either way, re-mint the baseline with `just dry-baseline` in the same commit.
- In tests, a fixture or setup function is the helper. A test still reads top
  to bottom when the helper's name says what it sets up.

== Out of scope

- Other clones in the same files. Each has its own task, or none.
- The gate's threshold and modes.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
