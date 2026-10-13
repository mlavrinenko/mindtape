#import "@local/mindtape:0.2.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "extract the clone at crates/mindtape-cli/src/cmd/import/tests.rs:246 (7 copies)",
  kind: "refactor",
  priority: ice(
    impact: 6,
    confidence: 0.6,
    ease: 9,
  ),
  difficulty: 2,
  estimate: (src-files: 3),
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
+ 🔴 This clone appears 7 times across 2 files, 109 jscpd tokens at most. Extracting it saves about 609 tokens net of the helper and its calls.
  - #raw("crates/mindtape-cli/src/cmd/import/tests.rs:160")
  - #raw("crates/mindtape-cli/src/cmd/import/tests.rs:196")
  - #raw("crates/mindtape-cli/src/cmd/import/tests.rs:246")
  - #raw("crates/mindtape-cli/src/cmd/import/tests.rs:265")
  - #raw("crates/mindtape-cli/src/cmd/import/tests.rs:306")
  - #raw("crates/mindtape-cli/src/cmd/import/tests.rs:375")
  - and 1 more, which `just ask-refactoring` lists again on its next run

The first copy reads:

```rust
        entries.is_empty(),
        "a failed record must not leave a stray file: {entries:?}"
    );
}

#[test]
fn status_default_flag_fills_a_missing_status() {
    let (dir, project) = setup_dest();
```

`just check-dry` fails a new clone, and `.jscpd-baseline.json` lists this one as
older than the gate. The arm joins jscpd's pairs into a family by shared
location, so a location may belong to a neighbouring window.

== TO-BE

- 🟢 The clone lives in one helper, or its copies carry a `jscpd:ignore`
  marker whose comment says why they match, and `just dry-baseline` has
  dropped it from the baseline.

== Changes

- `crates/mindtape-cli/src/cmd/import/tests.rs` and every other file the Problem lists: extract a helper and call it from each copy, or mark the copies as intended with `// jscpd:ignore-start` and `// jscpd:ignore-end` and a comment saying why the match is a coincidence. Either way, re-mint the baseline with `just dry-baseline` in the same commit.
- In tests, a fixture or setup function is the helper. A test still reads top
  to bottom when the helper's name says what it sets up.

== Out of scope

- Other clones in the same files. Each has its own task, or none.
- The gate's threshold and modes.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
