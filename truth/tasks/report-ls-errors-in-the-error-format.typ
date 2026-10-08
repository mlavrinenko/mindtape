#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report ls errors in the error format",
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

+ Run `mt ls --sort nofield`.
+ 🔴 It prints `Error: unknown sort field: nofield`.
+ Run `mt ls -i | cat`.
+ 🔴 It prints `Error: --interactive needs a TTY on stdin and stdout (not available when piped)`.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.

== Changes

- `crates/mindtape-cli/src/cmd/ls/mod.rs`: the sort error as a usage `Diag` naming the field.
- `crates/mindtape-cli/src/cmd/ls/rules.rs`, `crates/mindtape-cli/src/cmd/ls/interactive/mod.rs`: `Diag`s, no `{:?}`.
- `crates/mindtape-cli/src/render/columns.rs`: the unknown column with a did-you-mean.
- `crates/mindtape-cli/src/cli/check_argv.rs`: its refusal as a `Diag` rather than `Diag::legacy`; drop the `allow` and its `diag/allowlist.rs` entry for `ls` in `crates/mindtape-cli/src/cmd/mod.rs`.

== Out of scope

- The filter errors, already usage errors since the error DSL task.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
