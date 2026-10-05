#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Run just check in fail-fast stages and gate coverage and CRAP in it",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "coverage"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the CRAP ratchet is checked on every commit, and a lint failure shows in seconds],
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
  status: proposed(2026, 10, 6),
  links: depends-on(
    "generate-www-generated-once-per-tree-instead-of-once-per-docs-gate.typ",
  ),
)

== Problem

+ Edit a source file and run `just check`.
+ 🔴 A formatting error is reported only after every gate finished, about 100 s later.
+ 🔴 `check-crap` prints that it skipped, because the lcov is stale, so the CRAP baseline drifted for 222 commits unseen.


== TO-BE

- 🟢 `just check` runs a fast stage, then a slow stage, then `cover` and `crap`, stopping at the first red stage.
- 🟢 The pre-commit hook's `mmz --is-fresh --tag gate` asserts fresh coverage and a passing CRAP ratchet.


== Changes

- `Justfile`: `check` as three stage recipes; `cover` and `crap` replace `check-crap`.
- `.mmz/config.yaml`: gate-tag `cover` and `crap`, drop `check-crap`.
- `.just/scripts/check-crap.sh`: delete.
- `www/utils/gate-notes.typ`: entries for the new gates.
- `.github/workflows/ci.yml`: fold the coverage job into `check`.


== Rollback

Revert the commit; `just check` runs every gate in parallel again.

== Out of scope

- Mutation testing, which stays out of `just check`.
- The contributing docs. Their own task.


== Dogfood

Every commit in this repo passes through the staged `just check`.
