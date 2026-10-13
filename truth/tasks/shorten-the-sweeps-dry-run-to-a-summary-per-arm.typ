#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Shorten the sweep's dry run to a summary per arm",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: (),
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
      sound-codebase: helps[the sweep reads at a glance, so it gets run],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
)

== Problem

+ Run `just ask-refactoring`.
+ 🔴 It runs 14 minutes and prints 53486 lines: the full body of each of 682 findings, one `mt add --print` per finding at about a second each.

== TO-BE

- 🟢 The dry run prints one line per arm with its finding count, then that arm's top 3 findings by ICE score, one line each.
- 🟢 The bodies of the shown findings land in `target/ask-refactoring/<stem>.typ`, so a proposal is still read before it is filed.
- 🟢 `--write` files every finding, as before.

== Changes

- `contrib/ask-refactoring.sh`: collect the dry-run findings per arm, rank them, render only the top ones.
- `docs/src/contributing/signal-sweep.typ` and `www/utils/sweep-flag-notes.typ`: say what the dry run prints.

== Rollback

- Revert the commit. Nothing reads the dry-run output.

== Out of scope

- How many findings an arm files. The dry arm's 639 is its own task.

== Dogfood

This repo audits its tech debt with `just ask-refactoring` in one screen.
