#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Pin error-path cleanup with injected failures",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("tooling", "tests"),
  estimate: (src-files: 1),
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
      sound-codebase: helps[cleanup arms become reachable in-process],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 1),
)

== Problem

+ Read `crates/mindtape/src/fs_atomic.rs:31-42` (`write_atomic`'s cleanup arm).
+ 🔴 The cleanup arm (`remove_file(&tmp)`) is only reachable when a step after the temp file's creation fails, and no test made one fail: deleting `let _ = fs::remove_file(&tmp);` left the suite green.
+ The 2026-09-30 scoped sweep's survivor, `fs_atomic.rs:36:22 replace match guard !dir.as_os_str().is_empty() with true`, is a separate matter: it is an equivalent mutant, already excluded in `.cargo/mutants.toml`. The real gap was this untested arm.
+ Injection (`fail` 0.5.1 behind a `failpoints` feature) would pin it but costs a dependency, a feature and a production-code hook for one arm. Rejected as too dear.

== TO-BE

- 🟢 One test makes the final `fs::rename` fail with no new dependency, feature or production change: the target is an existing non-empty directory, and a file cannot be renamed over it.
- 🟢 It asserts `write_atomic` errors, the directory and its contents are untouched, and no `.tmp` sibling is left in the parent.
- 🟢 `#[cfg(unix)]` like its siblings.

== Changes

- `crates/mindtape/src/fs_atomic_tests.rs`: `a_failed_rename_removes_the_temp_file_and_leaves_the_target_untouched`.

== Rollback

Delete the test; nothing else changed.

== Out of scope

- Fault injection (`fail` crate, `failpoints` feature): rejected as too costly for one arm.
- Failures at the write or fsync steps: they share the arm the rename failure now pins, and the existing permission test already covers creation.
- Non-Unix: rename-over-a-directory semantics differ on Windows, so the test stays unix-only.

== Dogfood

Verified red: with `let _ = fs::remove_file(&tmp);` deleted the new test fails, leaving `.task.typ.<uuid>.tmp` beside `task.typ`; restored, it passes. The survivor is a killed mutant.
