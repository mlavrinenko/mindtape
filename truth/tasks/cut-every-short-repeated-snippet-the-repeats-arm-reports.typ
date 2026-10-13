#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Cut every short repeated snippet the repeats arm reports",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 40),
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
      sound-codebase: helps[each setup line lives in one test helper, so a change to it is one edit],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
)

== Problem

+ Source `contrib/ask-refactoring/repeats.sh`, run jscpd at 15 tokens over `crates` and pipe its report through `score_repeats` with `REPEAT_GAIN=300`.
+ 🔴 It prints 12 groups. The biggest is `let dir = tempfile::tempdir().unwrap();`, 395 copies in 107 files. The rest are test setups pasted 8 to 58 times: an `mt add` run and read-back, a `Condition::parse` assert, a link-edge fixture, a self-test re-exec, a rules-test project and a template const.

== TO-BE

- 🟢 The repeats arm reports no group at its default `--repeat-gain 300`.
- 🟢 Each extracted snippet is one test helper, and where a raw call has a single sanctioned door, `clippy.toml` bans it outside the helper.

== Changes

- `crates/mindtape/src/test_util.rs`, `crates/mindtape-cli/src/test_util.rs`, `crates/mindtape-cli/tests/it/support/mod.rs`: the shared helpers.
- `crates/**/*_tests.rs`, `crates/**/tests.rs`, `crates/mindtape-cli/tests/it/*.rs`: the copies call the helpers.
- `clippy.toml`: a `disallowed-methods` entry per locked helper.

== Rollback

- Revert the commits. Only test code and lint config change.

== Out of scope

- Groups under the gain bar, which the arm says are not worth a helper.
- Clones of 50 tokens and up, which `just check-dry` and its baseline own.

== Dogfood

This repo's `just ask-refactoring` files no repeats task.
