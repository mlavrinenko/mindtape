#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fix the sweep's retired estimate unit",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a dry run and a write agree on what the project's rules accept],
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
  status: proposed(2026, 9, 28),
)

== Problem

+ Run `just ask-refactoring --write` on a tree whose config renamed `[tasks.estimate] units` to `src-files`.
+ 🔴 The sweep dies on its first finding: `error: size-…-config-rules-rs: estimate: files is not an allowed estimate unit`, and `--write` files nothing after it.
+ `just ask-refactoring` dry exits 0 and prints the same task, because `mt add --print` returns before the validation every write runs.


== TO-BE

- 🟢 The four arms emit `(src-files: N)`, so the sweep validates and files every finding.
- 🟢 `mt add --print` runs the same pre-write check as a write, so a dry preview can never show a task the config would refuse.
- 🟢 A regression test pins the preview to the write's verdict.


== Changes

- `crates/mindtape-cli/src/add.rs`: validate the rendered task before the `--print` return, keeping `--no-verify` as the escape hatch.
- `crates/mindtape-cli/tests/add_format_e2e.rs`: a red-first test that a config-refused fill fails `--print` with the write's error and leaves no file.
- `contrib/ask-refactoring/size.sh`, `contrib/ask-refactoring/coverage.sh`, `contrib/ask-refactoring/mutants.sh`, `contrib/ask-refactoring/undertested.sh`: `(files: N)` to `(src-files: N)`, comments following.


== Rollback

Revert the commit; the sweep prints the retired unit again and its next dry run still exits 0.


== Out of scope

- The mutants arm's stale `mutants.out`: a full sweep is hours and stays a separate run.
- The estimate caps and the `src-files` definition: the rename landed already.


== Dogfood

Not applicable: this hardens the sweep this repo runs on its own tree.

