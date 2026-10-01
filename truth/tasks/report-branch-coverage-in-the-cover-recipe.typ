#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report branch coverage in the cover recipe",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 2,
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
      sound-codebase: helps[unexercised edges surface before a sweep, not during triage],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: cancelled(
    2026,
    10,
    1,
  )[cargo llvm-cov --branch is unstable (nightly-only) and rust-toolchain.toml pins stable 1.92.0.],
)

== Problem

+ Read `crates/mindtape/src/config_rules_eval.rs`'s `resolve_and_select` exclusion in `.cargo/mutants.toml`.
+ 🔴 `if skip > 0` was line-covered by every test; no test passed `skip == 0`, so the sweep's `>= 0` mutant was unkillable and had to be excluded by hand. Line coverage cannot see the difference.
+ Branch coverage (`cargo llvm-cov --branch`) reports each control-flow edge, so the unexercised false arm would show up before a sweep.

== TO-BE

- 🟢 `just cover` reports branch coverage alongside line coverage (or offers a documented flag that does).
- 🟢 The coverage deep dive names branch coverage as the cheap pre-sweep filter it is; the CRAP/coverage gate is unchanged.
- 🟢 lcov/generated artifacts carry the branch data or the run states it does not.

== Changes

- `.just/scripts/cover.sh`: add `--branch` (llvm-cov supports it; verify runtime cost) and, if the arm's lcov consumer cannot read branch records, keep the line report and print branch percentages separately.
- `docs/src/contributing/coverage-crap.typ` (with regenerated `.md`): one paragraph on when to read branch over line coverage.
- `Justfile` `cover` doc line updated.

== Rollback

Remove the flag from `cover.sh`; line coverage is exactly as today.

== Out of scope

- A branch-coverage threshold or gate: the signal stays advisory, like mutation testing.
- Replacing mutation testing: branch coverage sees unexercised edges, never unasserted lines.

== Dogfood

Before the next sweep, the `skip == 0` false arm shows uncovered, so the equivalent mutant is pre-empted or the code form is chosen knowingly.
