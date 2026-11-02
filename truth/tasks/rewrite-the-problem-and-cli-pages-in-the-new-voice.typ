#import "@local/mindtape:0.3.0": *
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
    product: verdicts(
      (
        cheap-adoption: helps[a newcomer understands what `mt` does and how to use it on the first read],
      ),
      "readable-tasks",
      "implementable-tasks",
      "enforced-practice",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (:),
      "sound-codebase",
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    7,
  )[problem pages, CLI index, command pages and cli-notes rewritten in the Voice register],
  links: depends-on("give-the-docs-a-conversational-voice.typ"),
)

== AS-IS

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
