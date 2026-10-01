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
  estimate: (src-files: 3),
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
  status: proposed(2026, 10, 1),
)

== Problem

+ Read `crates/mindtape/src/fs_atomic.rs:31-42` (`write_atomic`'s cleanup arm).
+ 🔴 `crates/mindtape/src/fs_atomic.rs:36:22 replace match guard !dir.as_os_str().is_empty() with true` and its sibling survived the 2026-09-30 scoped sweep: the cleanup arm (`remove_file(&tmp)`) is only reachable when a write/fsync/rename fails, and the normal suite cannot make it fail.
+ A verified demo in `/tmp/fail-demo` showed the recipe: `fail` 0.5.1 behind a `failpoints` Cargo feature, `fail::fail_point!` inside `write_and_rename`'s rename step, then `fail::cfg(RENAME_FP, "return")` and assertions that `write_atomic` errors, the target keeps its old bytes and no temp file remains. Feature-off, the injected test must skip (running it anyway fails on `unwrap_err()`, which proves the dependency).

== TO-BE

- 🟢 A `failpoints` feature enables the injection; `just test` / `just check` run with it on, so the cleanup arms are pinned in the gate.
- 🟢 Feature-off runs skip visibly; the mutation sweep runs with the feature too.
- 🟢 The injected assertions are the same ones a reader would want: error kind, target untouched, temp removed.

== Changes

- `Cargo.toml`: `fail` in `[workspace.dependencies]`, a `failpoints = ["dep:fail"]` feature on `mindtape`.
- `crates/mindtape/src/fs_atomic.rs`: a `#[cfg(feature = "failpoints")] fail::fail_point!` for the rename step in `write_and_rename`.
- `fs_atomic`'s test module: the injected rename-failure test, gated on the feature, skipping loudly without it.
- `.cargo/mutants.toml`: add the feature to `additional_cargo_args` so swept mutants face it.

== Rollback

Remove the feature and dependency; the injected tests skip again and the normal suite is unchanged.

== Out of scope

- Fault injection across the crate: only paths whose cleanup has no other coverage.
- Changing `fail` semantics or forking the demo project: `/tmp/fail-demo` is evidence, not code to import.

== Dogfood

Deleting `let _ = fs::remove_file(&tmp);` makes the injected test fail while the normal suite still passes, so the survivor is a killed mutant.
