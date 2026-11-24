#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Dedupe the copy-paste in the cmd tests outside add",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  kind: "dev",
  tags: ("stateless", "tests", "cli", "repeats"),
  assignee: "agent",
  status: done(
    2026,
    11,
    24,
  )[helpers extracted across eval, check, fix, graph and flip. 61 clones left the dry baseline],
)

== AS-IS

The `crates/mindtape-cli/src/cmd/` tests outside `add/` paste the same setup and assertion lines instead of calling a helper.

+ Run the repeats arm: `jscpd --no-tips -f rust -k 15 -l 1 -r json -o /tmp/rep crates`, then `score_repeats` from `contrib/ask-refactoring/repeats.sh`.
+ 🔴 It reports 9 snippets anchored here, 330 to 3633 tokens of gain: `eval_tests.rs:258` 77 copies, `eval_tests.rs:251` 43, `eval_tests.rs:311` 20, `flip/hooks_approval_tests.rs:72` 54, `fix/scope_tests.rs:37` 59, `flip/tests_output.rs:199` 23, `graph/tests.rs:61` 14, `check/tests.rs:28` 17, `flip/chain.rs:57` 24.
+ The anchor is one copy, and the real set is wider. `check/tests.rs` repeats `assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok())` 7 times and `.is_err()` 5, `flip/tests_output.rs` repeats the `rerun_isolated_stdout(FLIP_CHILD_ENV)` guard 6 times, `fix/scope_tests.rs` repeats `let dir = tempdir(); let root = dir.path();` 5 times, and three files restate one `TASK` marker.

== TO-BE

- 🟢 One helper in `crate::test_util` or the module's own test support holds each repeated snippet, and every copy calls it.
- 🟢 The tests assert what they asserted before: same fixtures, same expected text, same failures.
- 🟢 No `jscpd:ignore` marker hides a copy. Every copy calls the helper.

```rust
// cmd/check/tests.rs
fn assert_ok(files: &[(&str, &str)], config: Config) {
    let (_dir, project) = make_project_with_config(files, config);
    assert!(super::run_with(&project, &super::CheckArgs::default()).is_ok());
}
```

== Changes

- `crates/mindtape-cli/src/test_util.rs`: the shared `METADATA_TASK` source and `metadata_task_with`. The four marker fixtures derive from one constant.
- `crates/mindtape-cli/src/cmd/eval_tests.rs`: import the shared `TASK` instead of restating the marker.
- `crates/mindtape-cli/src/cmd/check/tests.rs`: `assert_ok`/`assert_err` over files and config, `comments_config`, and `task_with_comment`.
- `crates/mindtape-cli/src/cmd/check/tests_rules.rs`: `assert_ok`/`assert_err` over a config and a source, and the two task fixtures as functions.
- `crates/mindtape-cli/src/cmd/check/range_tests.rs`: `commit` and `ok_repo`.
- `crates/mindtape-cli/src/cmd/check/commit_tests.rs`: `committed` and `message_file`.
- `crates/mindtape-cli/src/cmd/graph/tests.rs`: `one_edge` and `external_task`.
- `crates/mindtape-cli/src/cmd/fix/scope_tests.rs`: `write_typs` and `scopes`.
- `crates/mindtape-cli/src/cmd/flip/tests.rs`: `flipped`, `refusal` and `assert_refused`.
- `crates/mindtape-cli/src/cmd/flip/tests_children.rs`, `tests_unblocked.rs`, `tests_note_required.rs`, `tests_revalidate.rs`, `tests_verbs.rs`, `tests_output.rs`: call the `flip/tests.rs` helpers.
- `crates/mindtape-cli/src/cmd/flip/hooks_tests.rs`: `visible`, `hidden` and `forced`.
- `crates/mindtape-cli/src/cmd/flip/hooks_approval_tests.rs`: `unallowed`.
- `crates/mindtape-cli/src/cmd/flip/hooks_after_tests.rs`: `after`.
- `crates/mindtape-cli/src/cmd/flip/batch_tests.rs`: `revalidated`.

== Out of scope

- `crates/mindtape-cli/src/cmd/add/`: another task owns it.
- `crates/mindtape/src/`: the library's own copies, outside this area.
- The repeats arm's scoring and the `lint::dry` gate: untouched.
- `flip/chain.rs`'s derive line: a derive attribute is idiomatic Rust, not a pasted idea, and no helper can replace it.

== Dogfood

Not applicable: the change is test-only, and this repo's own suite is its only consumer.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[one helper per repeated snippet means a fixture change lands once, and the suite still pins the same behavior].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
