#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the contributor docs in the new voice",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (src-files: 15),
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
  )[CONTRIBUTING and every contributor deep dive speak to the reader],
  links: depends-on("give-the-docs-a-conversational-voice.typ"),
)

== AS-IS

+ Read `CONTRIBUTING.md`, Worktrees.
+ 🔴 "It refuses when this checkout has uncommitted changes, because `git worktree add` copies commits only."

== TO-BE

- 🟢 CONTRIBUTING and every `docs/contributing/*.md` follow Voice in `docs/contributing/documentation.md`.

== Source of truth

- `docs/contributing/documentation.md`, Voice and Plain prose.

== Changes

- `docs/src/contributing.typ`, `docs/src/contributing/*.typ`, and their renders through `just docs md`.

== Out of scope

- AGENTS.md: the agent-docs task.

== Dogfood

This repo's docs are the change.
