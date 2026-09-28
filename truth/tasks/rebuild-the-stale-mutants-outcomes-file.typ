#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rebuild the stale mutants outcomes file",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
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
      sound-codebase: helps[a stale outcomes file cannot hide a survivor the suite fails to pin],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 28),
)

== Problem

+ Run `just ask-refactoring`.
+ 🔴 It prints `ask-refactoring: mutants.out/outcomes.json predates crates/mindtape/tests/dogfood_check.rs — skipping the mutants arm`: the file was last written 2026-09-10, so surviving mutants from every commit since are invisible to the sweep.
+ The skip names `just mutants-diff`, but on `main` that diffs against `main` — an empty diff, which measures nothing.


== TO-BE

- 🟢 `mutants.out/outcomes.json` postdates every `crates/**/*.rs`; the sweep runs the mutants arm again.
- 🟢 Every survivor the fresh run names is pinned by a test, filed as a task, or excluded with a reason in `.cargo/mutants.toml`.
- 🟢 The skip message names the command that actually refreshes the file.


== Changes

- `mutants.out/outcomes.json`: refreshed by a full `just mutants` (hours), or by `just mutants -f FILE` for each source changed since the last full run.
- `contrib/ask-refactoring/mutants.sh`: the skip message stops naming `just mutants-diff`, which cannot refresh on `main`.
- `truth/tasks/*`, tests or `.cargo/mutants.toml`: one remedy per survivor the run leaves unpinned.


== Rollback

Delete `mutants.out/`; the arm skips loudly again, as it does today.


== Out of scope

- CI mutation runs: `mutants-diff` stays the per-branch check.
- Survivors the stale file already names: this refresh starts a new measurement.


== Dogfood

The next `just ask-refactoring` reads a measurement instead of a skip.

