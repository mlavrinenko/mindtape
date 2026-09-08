#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "hint at the \\# escape when a task body fails to evaluate",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("cli", "docs"),
  assignee: "agent",
  estimate: (files: 3),
  status: done(2026, 9, 8),
)

== Summary

Task bodies are Typst markup, so a literal `\#N` in prose is read as a
variable reference and the task fails to load with `eval error: unknown
variable: N` — a message that names the variable but never the remedy.
The prose writer has to discover `\#` (or a code block) on their own.
This repo's own task `core-template-language.typ` in octrimmer hit exactly
this; the fix was a reader-side escape, which is invisible to the next
author who trips over the same wall.

`mt check` already reports the failure loudly, but loud is not instructive.

== Scope

- Mirror the existing per-run hint in `crates/mindtape-cli/src/install_hint.rs`
  (`missing_prelude_hint` derives one `Option<&'static str>` from the whole
  `LoadOutcome` and prints it once): add a sibling hint that fires when any
  `LoadOutcome` error message starts with `unknown variable`, answering
  "literal `#` in Typst markup? escape it as `\#`, or put the prose in a code
  block". Reuse the same once-per-run wiring so a 2000-task backlog reports it
  once, next to the existing prelude hint.
- Unit tests beside `missing_prelude_hint`'s: an error whose message is
  `unknown variable: N`, one whose message merely contains the phrase mid-text,
  and the existing prelude-path case proving both hints can coexist.
- Docs: one note in the writing-tasks documentation (quickstart or the task
  DSL page) — `#` starts an inline expression, so literal hashes in prose are
  written `\#`.

== Out of scope

Auto-escaping body prose before evaluation: bodies are intentionally real
Typst, and escaping would break the markup that currently works.
