#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Exclude split test files from the CRAP gate",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt", "tooling"),
  estimate: (files: 3),
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
      sound-codebase: helps[the CRAP gate scores production code only, so a baseline re-mint is a review of complexity that ships],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 9, 16),
)

== Problem

+ Run `just cover`, then `just crap`.
+ 🔴 `crap-baseline.json` carries 45 entries from 15 test files, among them `crates/mindtape/src/validate_tests_shape.rs` and `crates/mindtape/src/config_packs_tests_body.rs`.
+ Read `CRAP_EXCLUDES` in the `Justfile`.
+ 🔴 It drops `**/*_tests.rs` and `**/tests.rs` only, so a test module split under a longer name is scored as production code, and every baseline re-mint carries its helpers past a reviewer.


== TO-BE

- 🟢 `just crap` and `just crap-baseline` skip every test module, split ones included.
- 🟢 `crap-baseline.json` holds no entry from a file whose name carries `_tests`, and no production function's score moves.


== Changes

- `Justfile`: `CRAP_EXCLUDES` gains `--exclude **/*_tests_*.rs`, the one list both the gate and the baseline read.
- `crap-baseline.json`: re-mint with `just crap-baseline` in the same commit, since the scored set changes. The diff must only remove test-file entries; if a production entry moves, stop and say so.
- `docs/src/contributing/coverage-crap.typ`: name which files the gate skips, then `just docs md`.


== Rollback

Revert the commit: the exclude list and the baseline move back together, and nothing outside the repo changed.


== Out of scope

- The threshold and the ratchet arm: this changes what is measured, not where the bar sits.
- Test files that already match `*_tests.rs`: they are skipped today.


== Dogfood

Built in: this repo's own baseline loses the 45 test entries it carries today.

