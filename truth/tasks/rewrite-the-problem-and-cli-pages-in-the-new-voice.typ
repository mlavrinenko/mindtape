#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the problem and CLI pages in the new voice",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (src-files: 30),
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
  status: wip(2026, 10, 7),
  links: depends-on("give-the-docs-a-conversational-voice.typ"),
)

== Problem

+ Open `www/content/one-task-at-a-time.typ`.
+ 🔴 "Because a write command refuses a change that breaks a rule, `mt wip` refuses the second start."

== TO-BE

- 🟢 The problem pages, `www/content/cli/` and the notes in `www/utils/cli-notes.typ` follow Voice in `docs/contributing/documentation.md`.

== Source of truth

- `docs/contributing/documentation.md`, Voice and Plain prose.

== Changes

- The pages in the Problems nav group, `www/content/cli/*.typ`, `www/utils/cli-notes.typ`.

== Out of scope

- The `--help` text the CLI pages render: its own task.

== Dogfood

This repo's docs are the change.
