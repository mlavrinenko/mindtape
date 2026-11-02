#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add the sweep's size arm to the size fixture gate's inputs",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (src-files: 3),
  x-goals: (
    product: verdicts(
      (:),
      "readable-tasks",
      "implementable-tasks",
      "enforced-practice",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (
        sound-codebase: helps[a gate re-runs when the code it asserts changes],
      ),
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
)

== AS-IS

+ Commit `82070d9b`, which edits `contrib/ask-refactoring/size.sh`, then run `just check`.
+ 🔴 It passes with `just size-fixture-check` cached, though `bash .just/scripts/size-fixture-check.sh` fails: `score_size read crates/x/src/config/tests_id.rs as '', not a test`. The gate sources the size arm, but its mmz inputs omit it.

== TO-BE

- 🟢 An edit to `contrib/ask-refactoring/size.sh` makes `just size-fixture-check` stale.
- 🟢 The `.linecop.yaml` Shell comment and the undertested arm's cost comment match the tree.

== Changes

- `.mmz/config.yaml`: add the size arm to `script-size-fixture-check`.
- `.linecop.yaml`: the Shell comment names the two overrides.
- `contrib/ask-refactoring/undertested.sh`: the coverage arm now re-reads the lcov, not llvm-cov.

== Rollback

- Revert the commit.

== Out of scope

- Other gates' input lists; each is its own finding when one shows stale.

== Dogfood

This repo's `just check` catches a size arm change that breaks the fixture.
