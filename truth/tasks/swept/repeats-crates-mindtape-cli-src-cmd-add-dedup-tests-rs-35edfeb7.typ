#import "@local/mindtape:0.2.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "extract the snippet crates/mindtape-cli/src/cmd/add/dedup_tests.rs:68 repeats 21 times",
  kind: "refactor",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 17),
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
+ 🔴 This snippet repeats 21 times across 16 files, 21 jscpd tokens each. Extracting it saves about 305 tokens net of the helper and its calls.
  - #raw("crates/mindtape-cli/src/cmd/add/dedup_tests.rs:68")
  - #raw("crates/mindtape-cli/src/cmd/add/interactive_tests_editor.rs:104")
  - #raw("crates/mindtape-cli/src/cmd/add/tests.rs:340")
  - #raw("crates/mindtape-cli/src/cmd/add/values_tests_templates.rs:155")
  - #raw("crates/mindtape-cli/src/cmd/add/values_tests_templates.rs:160")
  - #raw("crates/mindtape-cli/src/cmd/add/values_tests_templates.rs:169")
  - #raw("crates/mindtape-cli/src/cmd/add/yaml_values_tests.rs:77")
  - #raw("crates/mindtape-cli/src/cmd/eval_tests.rs:136")
  - #raw("crates/mindtape-cli/src/cmd/eval_tests.rs:145")
  - #raw("crates/mindtape-cli/src/cmd/flip/tests_unblocked.rs:44")
  - #raw("crates/mindtape-cli/src/cmd/link/tests.rs:196")
  - #raw("crates/mindtape-cli/src/cmd/ls/interactive/tests.rs:272")
  - and 9 more, which `just ask-refactoring` lists again on its next run

The first copy reads:

```rust
    let err = resolve(DedupMode::Fail, "foo", "tasks/foo.typ", |_| true).unwrap_err();
    assert_eq!(
        crate::diag::plain(&err),
```

`just ask-refactoring` filed this from jscpd's pairs at 15 tokens, under the
50-token floor of `just check-dry`. The arm groups the pairs by hand until
jscpd's own clone classes ship (`canibump.toml`, `jscpd-clone-classes`), so a
location may belong to a neighbouring window of the same snippet.

== TO-BE

- 🟢 The snippet lives in one helper, and `just ask-refactoring` no longer
  names it.

== Changes

- `crates/mindtape-cli/src/cmd/add/dedup_tests.rs` and every other file the Problem lists: extract a helper and call it from each copy, or mark the copies as intended with `// jscpd:ignore-start` and `// jscpd:ignore-end` and a comment saying why the repetition is a coincidence.
- In tests, a fixture or setup function is the helper. A test still reads top
  to bottom when the helper's name says what it sets up.

== Out of scope

- Other snippets in the same files. Each has its own task, or none.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
