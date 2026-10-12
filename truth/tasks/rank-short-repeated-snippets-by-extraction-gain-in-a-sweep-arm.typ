#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rank short repeated snippets by extraction gain in a sweep arm",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 8),
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
      sound-codebase: helps[a snippet too short for the clone gate, pasted everywhere, reaches the backlog as one task],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    13,
  )[just ask-refactoring's repeats arm files short snippets by extraction gain],
)

== Problem

+ Run `just check-dry`. It passes, because its floor is 50 tokens.
+ 🔴 `let dir = tempfile::tempdir().unwrap();` and the setup after it repeat in 395 places across 78 test files. No gate or sweep reports it, because each copy is under 50 tokens.
+ jscpd can rank clone groups by size times copies only once upstream jscpd issue 1000 (clone classes) ships, and nothing here waits on it.


== TO-BE

- 🟢 `just ask-refactoring` has a `repeats` arm. It runs jscpd at 15 tokens, groups the pairs by snippet, drops `use`, `mod` and attribute lines, and scores each group by the tokens an extraction saves: `(copies - 1) × tokens - 5 × copies - 10`.
- 🟢 Groups that mostly overlap a higher-scoring one fold into it, so one snippet files one task.
- 🟢 `--repeat-gain N` sets the bar, 300 by default, which files 12 tasks on today's tree.
- 🟢 The arm says it is a stopgap, and `canibump.toml` says when jscpd's own clone classes can replace it.


== Changes

- `contrib/ask-refactoring/repeats.sh`: the arm, its scoring and the reasoning behind its numbers.
- `contrib/ask-refactoring.sh`: `repeats` in `ARMS`, and the `--repeat-gain` flag.
- `Justfile`: the flag's `[arg(...)]` and default on `ask-refactoring`.
- `.mindtape/templates/sweep-repeats.typ`: the task a group becomes.
- `.mindtape/config.toml`: a `repeats` tag beside `size`, `mutants` and `coverage`.
- `www/utils/sweep-flag-notes.typ`: why the bar is its own number.
- `outdatty.yaml`: the arm and its template join `sweep-templates`.
- `docs/src/contributing/signal-sweep.typ`: the arm, and why jscpd counts as a gate only from 50 tokens up.


== Rollback

- Revert the commit. The sweep loses the arm, and the tasks it filed stay.


== Out of scope

- Gating on short repeats, which needs a stable group identity that only jscpd's clone classes give.
- The canibump setup, which is its own task.


== Dogfood

This repo's `just ask-refactoring --write` files the first batch.
