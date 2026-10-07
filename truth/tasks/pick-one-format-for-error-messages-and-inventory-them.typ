#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Pick one format for error messages and inventory them",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("docs",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a newcomer understands what `mt` does and how to use it on the first read],
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
  links: depends-on("give-the-docs-a-conversational-voice.typ"),
)

== Problem

+ Run `mt wip nonexistent-xyz`.
+ 🔴 It prints `error: nonexistent-xyz: no task with id "nonexistent-xyz"` and then `Error: 1 task(s) had problems`.
+ Run `mt ls 'foo('`.
+ 🔴 It prints `Error: failed to parse filter: Found an unmatched opening parenthesis '('.`

== TO-BE

- 🟢 Each error is one line that starts with `error:`, quotes the value that failed, and names the next step when there is one.

== Source of truth

- `docs/contributing/documentation.md`, Voice and Plain prose.

== Changes

- An inventory of every message `mt` prints, grouped by where it is built, and one format for all of them, shown on three real messages.
- A follow-up task for the mechanical rewrite of the rest.

== Out of scope

- `--help` text: its own task.

== Dogfood

This repo's docs are the change.
