#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Measure the sweep's per-file coverage from lcov",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("found", "tooling"),
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
      sound-codebase: helps[the coverage arm stops filing files the suite already covers],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
)

== Problem

+ Running `just ask-refactoring` after the first sweep batch landed
+ 🔴 #raw("contrib/ask-refactoring/coverage.sh"): the coverage arm reads `cargo llvm-cov report --json --summary-only`, which counts a line once per monomorphized copy of the binary crate, so all 9 files it reports sit at or above the floor in `lcov.info`

```
crates/mindtape-cli/src/cmd/flip/hooks_cmd.rs   json  75.0  lcov 100.0
crates/mindtape-cli/src/diag/io.rs              json  84.6  lcov 100.0
crates/mindtape-cli/src/cmd/preset.rs           json  79.7  lcov  93.2
crates/mindtape-cli/src/cmd/review.rs           json  79.1  lcov  88.1
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("contrib/ask-refactoring/coverage.sh").

== TO-BE

- 🟢 The coverage arm scores each file by the `LF`/`LH` lines of the fresh `target/coverage/lcov.info`, and reports none of the 9.

== Changes

- `contrib/ask-refactoring/coverage.sh`: build the per-file report from `lcov.info` instead of `cargo llvm-cov report --json`.

== Rollback

- Revert the commit.

== Out of scope

- The undertested arm, which reads `cargo crap` over the same lcov.

== Dogfood

This repo's sweep files only files the suite really leaves dark.
