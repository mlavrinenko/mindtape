#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Give the docs a conversational voice",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("docs",),
  estimate: (src-files: 4),
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
)

== Problem

+ Read the README intro or any page under `www/content/`.
+ 🔴 It reads like a spec: no "you", no contractions, and sentences such as "Because a write command refuses a change that breaks a rule, `mt wip` refuses the second start."
+ 🔴 The tagline says "your team's rules", though nothing in `mt` assumes a team.

== TO-BE

- 🟢 `docs/contributing/documentation.md` states the voice each reader gets: peer-to-peer for humans, imperative for agents, one-line errors.
- 🟢 The tagline, the README intro and the comparison page name no team.

== Source of truth

- `docs/contributing/documentation.md`, Voice and Plain prose.

== Changes

- `docs/src/contributing/documentation.typ`: a Voice section before Plain prose.
- `www/utils/site.typ`, `www/utils/demo.typ`, `www/content/comparison.typ`: the tagline, the intro and the comparison opener.

== Out of scope

- The other pages. Each area has its own rewrite task.

== Dogfood

This repo's docs are the change.
