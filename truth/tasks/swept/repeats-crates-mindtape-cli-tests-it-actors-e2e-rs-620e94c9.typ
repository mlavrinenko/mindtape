#import "@local/mindtape:0.2.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "extract the snippet crates/mindtape-cli/tests/it/actors_e2e.rs:126 repeats 24 times",
  kind: "refactor",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 16),
  assignee: "agent",
  tags: ("swept", "repeats"),
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
      sound-codebase: helps[a snippet pasted everywhere lives in one helper, so
        a fix lands once],
      docs-match-code: neutral,
    ),
  ),
  status: done(2026, 10, 13),
)

== Problem

+ Run `just ask-refactoring`.
+ 🔴 This snippet repeats 24 times across 15 files, 19 jscpd tokens each. Extracting it saves about 307 tokens net of the helper and its calls.
  - #raw("crates/mindtape-cli/tests/it/actors_e2e.rs:126")
  - #raw("crates/mindtape-cli/tests/it/add_assignee_e2e.rs:53")
  - #raw("crates/mindtape-cli/tests/it/add_dedup_e2e.rs:144")
  - #raw("crates/mindtape-cli/tests/it/add_dedup_e2e.rs:168")
  - #raw("crates/mindtape-cli/tests/it/add_e2e.rs:298")
  - #raw("crates/mindtape-cli/tests/it/add_file_operator_e2e.rs:145")
  - #raw("crates/mindtape-cli/tests/it/add_inherited_choices_e2e.rs:96")
  - #raw("crates/mindtape-cli/tests/it/add_inherited_choices_e2e.rs:109")
  - #raw("crates/mindtape-cli/tests/it/allow_e2e.rs:87")
  - #raw("crates/mindtape-cli/tests/it/allow_e2e.rs:126")
  - #raw("crates/mindtape-cli/tests/it/flip_hooks_e2e.rs:29")
  - #raw("crates/mindtape-cli/tests/it/flip_hooks_e2e.rs:273")
  - and 12 more, which `just ask-refactoring` lists again on its next run

The first copy reads:

```rust
    let dir = scaffold("");
    support::run_ok(dir.path(), &["check"]);
```

`just ask-refactoring` filed this from jscpd's pairs at 15 tokens, under the
50-token floor of `just check-dry`. The arm groups the pairs by hand until
jscpd's own clone classes ship (`canibump.toml`, `jscpd-clone-classes`), so a
location may belong to a neighbouring window of the same snippet.

== TO-BE

- 🟢 The snippet lives in one helper, and `just ask-refactoring` no longer
  names it.

== Changes

- `crates/mindtape-cli/tests/it/actors_e2e.rs` and every other file the Problem lists: extract a helper and call it from each copy, or mark the copies as intended with `// jscpd:ignore-start` and `// jscpd:ignore-end` and a comment saying why the repetition is a coincidence.
- In tests, a fixture or setup function is the helper. A test still reads top
  to bottom when the helper's name says what it sets up.

== Out of scope

- Other snippets in the same files. Each has its own task, or none.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
