#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Dedupe the copy-paste in the CLI end-to-end tests",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 2,
  kind: "dev",
  tags: ("stateless", "swept", "repeats", "tests"),
  assignee: "agent",
  status: done(2026, 11, 24),
)

== AS-IS

+ `crates/mindtape-cli/tests/it/` pastes the same scaffolding into every test file. A fix to the shape lands in one copy only.
+ 🔴 `just advisory ask-refactoring` reports 10 snippets at 15 tokens with gain >= 300. The largest is 88 copies of `support::mt(dir.path())` (21 tokens, gain 1377).
+ 🔴 `support::mt(dir.path()).args([...]).output().unwrap()` opens 40 tests. `run_stderr` plus `assert_eq!(code, Some(1))` follows in 30.
+ 🔴 `fn shown` wraps `native_in` in 20 files. `assert!(dir.path().join("...").exists())` appears in 27 places.
+ 🔴 Findings, by first copy: `actors_e2e.rs:133` 88, `actors_e2e.rs:132` 23, `allow_e2e.rs:250` 41, `allow_e2e.rs:250` 15, `allow_e2e.rs:52` 17, `add_dedup_e2e.rs:167` 53, `add_describe_title_e2e.rs:9` 26, `add_flags_e2e.rs:54` 15, `add_format_e2e.rs:334` 15, `date_pin_e2e.rs:185` 21.

== TO-BE

- 🟢 `support` holds one helper per repeated shape: `run`, `refusal`, `failed`, `listed`, `add_then_done`, `scaffold_with_existing_dup`, `scaffold_files`, `assert_exists` and `shown`.
- 🟢 Every copy calls the helper. A change to the shape lands once.
- 🟢 `just advisory ask-refactoring` finds nothing in `crates/mindtape-cli/tests/it/` at gain >= 300.

```
$ just advisory ask-refactoring
# no repeats finding under crates/mindtape-cli/tests/it/
```

== Changes

- `crates/mindtape-cli/tests/it/support/run.rs`: the run and assertion helpers, moved out of `mod.rs` so it stays under its linecop cap.
- `crates/mindtape-cli/tests/it/support/mod.rs`: re-export them, and add the fixture helpers the test files share.
- `crates/mindtape-cli/tests/it/*.rs`: call the helper from every copy, without changing behavior.

== Out of scope

- The other 18 findings outside `crates/mindtape-cli/tests/it/`. Each has its own task.
- The repeats arm, the dry gate and the first-line convention.

== Dogfood

Not applicable: a test-only refactor doesn't change a feature this repo could use.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a test helper that lives once cannot drift from its copies, and a fix to the shape lands in one place].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
