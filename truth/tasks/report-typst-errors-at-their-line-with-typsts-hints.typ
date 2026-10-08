#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report Typst errors at their line with Typst's hints",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("cli",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[every error says what failed and what to try, in one shape],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 8),
  links: depends-on(
    "build-an-error-dsl-that-fixes-the-message-format-and-the-exit-code.typ",
  ),
)

== Problem

+ Misspell `title` as `titel` in a task file and run `mt check`.
+ 🔴 It prints `warning: tasks/broken.typ:2: eval error: unexpected argument: titel`: a warning under the command that fails on it, an `eval error:` prefix, no column, and none of Typst's hints.
+ Run `mt ls` on a project whose prelude path is wrong.
+ 🔴 It prints one `file not found` line per task.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 `mt check` prints `error: tasks/broken.typ:2:3: unexpected argument: titel`, then each Typst hint as `hint:`. `mt ls` prints the same line as `warning:`, because the listing still prints.
- 🟢 A Typst warning on a file that loads prints as its own `warning:` line, with its place.
- 🟢 A missing prelude is one error naming the config key.

== Changes

- `crates/mindtape/Cargo.toml`, `crates/mindtape-cli/Cargo.toml`: bump `typst-world` to 0.5.0 and `typst-harvest` to 0.6.0, which keep Typst's hints and the column. `Cargo.lock` follows. `HarvestError::Eval(err)` became `HarvestError::Eval { error, warnings }`: render each of `warnings` as a `warning:` line.
- `crates/mindtape/src/loader/mod.rs`: match the new `Eval` shape; carry the column, the hints and the warnings on `LoadError`; collapse a missing prelude into one error.
- `crates/mindtape-cli/src/util.rs`: the load-error lines as `Diag`s with `at(path, line, col)` and one `hint:` per Typst hint, with no `eval error:` prefix.
- `www/generate-task-files.sh`: three `check_invalid` lines (34, 35 and 39) grep for `<file>:<line>: eval error: <message>`; change them to the new line, which drops `eval error:` and adds the column.

== Out of scope

- Typst's own wording, which is kept verbatim.
- `cmd/check/templates.rs`, `cmd/install/hint.rs` and the `util` entry on `diag/allowlist.rs`: `report-template-load-failures-and-install-hints-in-the-error-format` does them.

== Dogfood

Every `mt` error in this repo's sessions about a broken task file comes out in the new shape, with Typst's hints.
