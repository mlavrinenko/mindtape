#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report check problems on stderr in the error format",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("cli",),
  estimate: (src-files: 3),
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
  status: proposed(2026, 10, 7),
  links: depends-on(
    "build-an-error-dsl-that-fixes-the-message-format-and-the-exit-code.typ",
  ),
)

== Problem

+ Run `mt check 2>/dev/null` in a project with a rule breach.
+ 🔴 The breach prints on stdout, and only `Error: 1 violation(s), 0 load error(s)` reaches stderr.
+ Run `mt check --commits HEAD~5..HEAD` in a repository with fewer commits.
+ 🔴 It prints `Error: --commits reads the range HEAD~5..HEAD`, a blank line, `Caused by:` and git's text.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 Every breach goes to stderr, a load error under `mt check` is `error:`, and the last line is `error: 3 problems in 2 tasks`. `ok:` stays on stdout.

== Changes

- `crates/mindtape-cli/src/cmd/check/mod.rs`: breaches and load errors to stderr through `emit`; the summary as one `Diag`.
- `crates/mindtape-cli/src/cmd/check/commit.rs`: commit-message problems to stderr.
- `crates/mindtape-cli/src/cmd/check/range.rs`: `error: can't read commit range: 'HEAD~5..HEAD': <git's reason>`.
- `crates/mindtape-cli/src/cmd/mod.rs`: drop the `allow` and its `diag/allowlist.rs` entry for `check` once `templates.rs` is converted too.

== Out of scope

- `cmd/check/templates.rs`: the Typst diagnostics task.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
