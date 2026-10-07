#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Build an error DSL that fixes the message format and the exit code",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the compiler and clippy hold the error format, not a review],
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
    "pick-one-format-for-error-messages-and-inventory-them.typ",
  ),
)

== Problem

+ Run `mt wip nonexistent-xyz`.
+ 🔴 It prints `error: nonexistent-xyz: no task with id "nonexistent-xyz"` and then `Error: 1 task(s) had problems`: two prefixes, the value twice, a summary for one task.
+ Run `mt ls 'status == wip'; echo $?`.
+ 🔴 It prints `Error: unknown field in filter: wip` and exits 1, the same code as a rule refusing work.

== TO-BE

- 🟢 Every message `mt` writes to stderr goes through one renderer and starts with `error:`, `hint:`, `note:` or `warning:`.
- 🟢 A message is built with a typed builder: the message text is a `&'static str`, and a value the user typed goes through `.value(..)`, which single-quotes it. A value cannot be formatted into the text.
- 🟢 Each error has a kind: usage (exit 2) or refused/failed (exit 1). The exit code comes from the kind, so no command picks a number.
- 🟢 `main` returns an `ExitCode`, so Rust's `Error:` and `Caused by:` output cannot appear, and clippy refuses `eprintln!` outside the renderer.

== Changes

- `crates/mindtape-cli/src/diag/`: the builder, the kinds, the renderer and its tests, red first.
- `crates/mindtape-cli/src/main.rs`: render the top-level error and exit with its kind's code.
- `clippy.toml`: disallow `eprintln!` and `anyhow::bail!` outside the renderer.
- `CONTRIBUTING.md` (source `docs/src/contributing.typ`) and `docs/src/contributing/documentation.typ`: the error rule points at the builder.
- The format itself is the Changes section of `pick-one-format-for-error-messages-and-inventory-them`.

== Alternatives

- Rules and review only. A rule nobody checks drifts; the current mix of `error:` and `Error:` is that drift.
- A `format!`-style macro. It keeps values inside free text, so nothing stops an unquoted value.

== Out of scope

- Converting every existing message. The fan-out tasks do that after this lands.
- Typst diagnostics, which wait on `typst-world` keeping hints.

== Dogfood

Every `mt` error in this repo's sessions comes out in the new shape once the fan-out lands.
