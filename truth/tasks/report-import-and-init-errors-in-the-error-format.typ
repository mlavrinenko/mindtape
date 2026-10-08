#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report import and init errors in the error format",
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

+ Run `mt import bad.json` on a file that isn't JSON.
+ 🔴 It prints `Error: parse canonical task-JSON`, a blank line, `Caused by:` and `expected ident at line 1 column 2`.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 `error: bad.json:1:2: not JSON: expected ident`.

== Changes

- `crates/mindtape-cli/src/cmd/import/mod.rs`, `crates/mindtape-cli/src/cmd/import/input.rs`: one line with the file and place; the skip count stays a summary.
- `crates/mindtape-cli/src/cmd/init/compose.rs`, `crates/mindtape-cli/src/cmd/init/preset.rs`, `crates/mindtape-cli/src/cmd/init/wizard/mod.rs`: shorten the long messages; drop the `allow`s and their `diag/allowlist.rs` entries for `import` and `init` in `crates/mindtape-cli/src/cmd/mod.rs`.

== Out of scope

- `cmd/init/mod.rs`'s I/O sites: they switch to the helper from `report-task-references-and-project-discovery-in-the-error-format`.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
