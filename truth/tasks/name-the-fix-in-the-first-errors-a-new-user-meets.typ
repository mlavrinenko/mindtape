#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the fix in the first errors a new user meets",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli", "mt"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a new user recovers from a missing setup step by reading the error, with no docs page],
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
)

== Problem

+ In a fresh home where `mt install` never ran, run `mt init --no-input`, then `mt add "Fix login"`.
+ 🔴 It prints `Error: the task as filled did not load: eval error: file not found (searched at <data>/typst/packages/local/mindtape/0.2.0/typst.toml)`. Nothing says `mt install` puts the prelude there.
+ Append `[taks]` and `x=1` to `.mindtape/config.toml`, then run `mt ls`.
+ 🔴 It prints ``unknown field `taks`, expected one of …`` twice, as cause 0 and cause 1, and names no line.
+ Run `mt ls` outside any project.
+ 🔴 It prints `not inside a MindTape project (no .mindtape/ directory found)` and names neither `mt init` nor `-w`.


== Environment

- `mt` built from `main` at the commit that filed this, on Linux, with `HOME` and the `XDG_*` directories pointed at an empty temporary directory.


== TO-BE

- 🟢 When a task file cannot find the `@local/mindtape` package, the error says to run `mt install`, or to pin the prelude under `[[typst.local-packages]]`.
- 🟢 A config error prints each message once, with the line and column when the TOML parser gives one.
- 🟢 Outside a project, the error names `mt init` to make one here and `-w <path>` to use another.


== Changes

- `crates/mindtape-cli/tests/first_run_errors.rs`, new: the red tests first, one per error.
- The module that wraps a task file's eval error: add the `mt install` hint on a missing `@local/mindtape` package.
- The config loader: stop repeating the cause, and keep the span.
- The project discovery error: name `mt init` and `-w`.


== Out of scope

- Errors that already name their fix.
- A docs page of errors. The fix belongs in the message.


== Dogfood

Not applicable: this repo's dev shell runs `mt install`, so its contributors never meet the missing-prelude error.

