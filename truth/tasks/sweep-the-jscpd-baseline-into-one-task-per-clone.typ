#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Sweep the jscpd baseline into one task per clone",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (src-files: 5),
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
      sound-codebase: helps[each clone the gate tolerates reaches the backlog as a task with its own scope, so the baseline shrinks instead of aging],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
  links: depends-on(
    "gate-on-new-jscpd-clones-renamed-and-near-miss-copies-included.typ",
  ),
)

== Problem

+ Run `jq '.fingerprints | length' .jscpd-baseline.json`.
+ 🔴 It prints about 2000: clones the gate tolerates because they predate it. Most are in tests, such as repeated setup in `crates/mindtape-cli/src/cmd/add/tests_subprocess.rs`.
+ 🔴 `just ask-refactoring` files none of them. Its header rules jscpd out as a signal a gate already enforces, which no longer holds for a baselined clone.


== TO-BE

- 🟢 `just ask-refactoring` files one task per baselined clone family, naming every location and the shared shape.
- 🟢 A task closes by extracting the clone, or by a `jscpd:ignore-start` marker whose comment says why the match is a coincidence, such as a lookup table matching itself one row down in `crates/mindtape/src/filter/expr.rs`. Either way the commit re-mints the baseline with `just dry-baseline`.


== Changes

- `contrib/ask-refactoring/dry.sh`: `signal_dry` reads the clones jscpd's JSON report marks as not new, and `emit_dry` files one task per clone family.
- `contrib/ask-refactoring.sh`: `dry` joins `ARMS`, and the header names the baseline as the part of jscpd no gate enforces.
- `.mindtape/templates/sweep-dry.typ`: the task a clone family becomes.
- `.mindtape/config.toml`: a `dry` tag beside `size`, `mutants` and `coverage`.
- `docs/src/contributing/signal-sweep.typ`: the new arm.


== Rollback

- Revert the commit. The sweep loses the arm, and the tasks it filed stay as they are.


== Out of scope

- The gate's threshold and modes, set by the task that made the baseline.
- Short snippets under the 50-token floor, which wait on upstream jscpd issue 1000 (clone classes).


== Dogfood

This repo's `just ask-refactoring --write` files the first batch.
