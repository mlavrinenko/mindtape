#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print only the total row of the coverage report",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("coverage", "tooling"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a gate run spends its output on the number it gates],
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
  status: done(2026, 10, 6),
)

== Problem

+ Run `just cover`.
+ 🔴 It prints a per-file table of about 230 lines after the test summary.


== TO-BE

- 🟢 `just cover` prints the header and the `TOTAL` row, and still fails below 90% of lines.


== Changes

- `.just/scripts/cover.sh`: keep the first and last line of the final `cargo llvm-cov report`; `--summary-only` drops nothing in text mode.


== Rollback

Revert the commit to print the full per-file table again.

== Out of scope

- The lcov report, which `just crap` reads.


== Dogfood

This repo's `just cover` and `just check-full` print two lines of coverage.
