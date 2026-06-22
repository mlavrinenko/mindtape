#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Receive task title from stdin in `mindtape add`",
  status: done(2026, 6, 22),
  priority: ice(impact: 5, confidence: 0.8, ease: 8),
  relates-to: (
    link("add-slots-from-env.typ"),
    link("add-interactive-fill.typ"),
  ),
  tags: ("cli",),
)

== Summary

`echo "hello world" | mindtape add` should file `hello-world.typ`: read the
piped text as the `title` slot and slug it into the filename stem. Today the
only stdin consumer is `-f -` (JSON values); a bare piped string is ignored and
`add` errors on the unfilled required `title`. Closing this makes the one-liner
the fast path for filing a task from a shell or another tool's output.

stdin is shared: `-f -` already reads it as JSON, and the interactive editor
keys off a stdin TTY. A piped title must therefore be gated — read stdin as a
title only when stdin is not a TTY and `-f -` was not passed — so it never
collides with those paths.

== Scope

- Read stdin as the title only when: stdin is not a TTY (piped/redirected), `-`
  is not among `-f/--values`, and `title` is not already set by a
  higher-precedence source. Trim, take the first line (title is single-line);
  empty input sets nothing.
- Precedence: `--set` > `-f` > `MINDTAPE_ADD_*` env > stdin title > slot default
  (env rank lands with add-slots-from-env.typ).
- Feed the existing `output_stem` -> `slug` -> write/verify path unchanged; the
  stem is `slug(title)` exactly as for `--set title=`.
- Keep the read in CLI glue; factor a pure helper (raw text -> `Option<String>`)
  so it is testable without touching process stdin.

== Notes

- No new dependency: `std::io::IsTerminal` is already used by the interactive
  gate; reuse it.
- Touches `crates/mindtape-cli/src/add.rs` (run) and `add_values.rs` (helper +
  tests). No library churn; `mindtape` stays format-blind.
- README usage block gains an `echo "title" | mindtape add` line.
