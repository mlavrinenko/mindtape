#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the agent docs for any agent",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (src-files: 8),
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
  status: done(
    2026,
    10,
    7,
  )[agent docs give one action per situation; human pages already leave mt as a choice],
  links: depends-on("give-the-docs-a-conversational-voice.typ"),
)

== Problem

+ Read `skills/mindtape/SKILL.md` and `README.md`.
+ 🔴 Both tell the reader to make every change through `mt`. A person may edit a task file as text, and only an agent needs the hard rule.

== TO-BE

- 🟢 AGENTS.md, SKILL.md and the skill's references follow the agent register in `docs/contributing/documentation.md`, for any agent.
- 🟢 The rule to change tasks through `mt` lives only in the agent docs. The human docs present `mt` commands as a shortcut beside editing the text.

== Source of truth

- `docs/contributing/documentation.md`, Voice and Plain prose.

== Changes

- `docs/src/agents.typ`, `docs/src/skill.typ`, `skills/mindtape/references/*.md`, and any human page that states the rule.

== Out of scope

- The skill's evals, unless a rewrite breaks one.

== Dogfood

This repo's docs are the change.
