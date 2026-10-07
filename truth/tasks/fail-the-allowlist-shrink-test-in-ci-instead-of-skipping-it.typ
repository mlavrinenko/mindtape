#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fail the allowlist shrink test in CI instead of skipping it",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tests",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the only-shrinks rule is checked in CI, not skipped],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 8),
)

== Problem

+ Run the `windows` job of `.github/workflows/ci.yml`, which checks out at depth 1.
+ 🔴 `the_list_only_shrinks_from_the_branch_point` in `crates/mindtape-cli/src/diag/allowlist_tests.rs` finds no `main` and returns, so it passes without checking anything.

== TO-BE

- 🟢 The `windows` job fetches full history, so the test has a `main` to compare with.
- 🟢 With `CI` set, a missing `main` or a missing list at the branch point fails the test; elsewhere it still skips.

== Changes

- `crates/mindtape-cli/src/diag/allowlist_tests.rs`: fail instead of return when `CI` is set.
- `.github/workflows/ci.yml`: `fetch-depth: 0` on the `windows` job checkout.

== Rollback

- Revert the commit.

== Out of scope

- The `check` job, which already fetches full history.

== Dogfood

This repo's CI runs the test for real on both jobs.
