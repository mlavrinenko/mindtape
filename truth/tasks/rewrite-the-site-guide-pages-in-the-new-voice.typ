#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the site guide pages in the new voice",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (src-files: 20),
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

+ Open `www/content/templates.typ`.
+ 🔴 "When a refused add left a field to a slot, another template writes that field as the value you passed, and a breach that refused the add is about that field, the error ends with a line such as ..."

== TO-BE

- 🟢 Each guide page under `www/content/` follows Voice in `docs/contributing/documentation.md`, and a reader gets it on the first pass.

== Source of truth

- `docs/contributing/documentation.md`, Voice and Plain prose.

== Changes

- Every `www/content/*.typ` page that is not a problem page, and `www/content/index.typ`.

== Out of scope

- The problem pages and `www/content/cli/`: their own task.

== Dogfood

This repo's docs are the change.
