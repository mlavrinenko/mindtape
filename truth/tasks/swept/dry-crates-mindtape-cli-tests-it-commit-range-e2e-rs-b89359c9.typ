#import "@local/mindtape:0.2.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "extract the clone at crates/mindtape-cli/tests/it/commit_range_e2e.rs:66 (9 copies)",
  kind: "refactor",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 8,
  ),
  difficulty: 2,
  estimate: (src-files: 9),
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
  status: proposed(2026, 10, 13),
)

== Problem

+ Run `just ask-refactoring`.
+ 🔴 This clone appears 9 times across 8 files, 68 jscpd tokens at most. Extracting it saves about 489 tokens net of the helper and its calls.
  - #raw("crates/mindtape-cli/src/cmd/check/range_tests.rs:63")
  - #raw("crates/mindtape-cli/tests/it/check_templates_e2e.rs:32")
  - #raw("crates/mindtape-cli/tests/it/commit_amend_e2e.rs:46")
  - #raw("crates/mindtape-cli/tests/it/commit_range_e2e.rs:66")
  - #raw("crates/mindtape-cli/tests/it/commit_range_e2e.rs:67")
  - #raw("crates/mindtape-cli/tests/it/estimate_rules_e2e.rs:77")
  - and 3 more, which `just ask-refactoring` lists again on its next run

The first copy reads:

```rust
        .args(["check", "--commits", range])
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
```

`just check-dry` fails a new clone, and `.jscpd-baseline.json` lists this one as
older than the gate. The arm joins jscpd's pairs into a family by shared
location, so a location may belong to a neighbouring window.

== TO-BE

- 🟢 The clone lives in one helper, or its copies carry a `jscpd:ignore`
  marker whose comment says why they match, and `just dry-baseline` has
  dropped it from the baseline.

== Changes

- `crates/mindtape-cli/tests/it/commit_range_e2e.rs` and every other file the Problem lists: extract a helper and call it from each copy, or mark the copies as intended with `// jscpd:ignore-start` and `// jscpd:ignore-end` and a comment saying why the match is a coincidence. Either way, re-mint the baseline with `just dry-baseline` in the same commit.
- In tests, a fixture or setup function is the helper. A test still reads top
  to bottom when the helper's name says what it sets up.

== Out of scope

- Other clones in the same files. Each has its own task, or none.
- The gate's threshold and modes.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
