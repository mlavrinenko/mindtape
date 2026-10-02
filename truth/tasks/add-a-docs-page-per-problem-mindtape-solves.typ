#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add a docs page per problem MindTape solves",
  kind: "docs",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("docs", "www"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[an engineer finds the config for their problem on one page],
      typst-over-markdown: helps[each page ends on `mt` refusing a bad task, which a Markdown tracker cannot do],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
  links: depends-on(
    "capture-the-api-service-examples-problem-solver-runs-for-the-docs.typ",
    "write-plain-prose-rules-for-the-docs-and-check-for-history-and-shouting.typ",
  )
    + parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Open the docs site and look for how to stop agents from filing tasks that touch 100 files.
+ 🔴 The NAV lists features: Task DSL, Filter language, Configuration. The answer is an `estimate` unit, a `[[tasks.rules]]` assert and `changed()`, spread over /estimates/, /filters/ and /config/. A reader must already know the mechanism to find it.
+ The same holds for the other four main problems: the required task shape, the done gate, one task in progress in dependency order, and the commit trail.


== TO-BE

- 🟢 The first NAV group is "Problems", with one page each: split oversized tasks; require a task shape; gate done on your checks; one task at a time, in order; trace commits to tasks.
- 🟢 Each page has the problem in one short paragraph, the config that solves it (a slice of `examples/api-service`), the capture of `mt` refusing or answering, and links to the reference pages.
- 🟢 Each page is under 60 source lines and follows the plain-prose rules.


== Source of truth

- `examples/api-service` and its captures in `www/generated/`.
- `crates/mindtape/src/packs/mindtape-tasks.toml`: the task shape and commit rules.


== Changes

- `www/content/split-oversized-tasks.typ`, `require-a-task-shape.typ`, `gate-done-on-your-checks.typ`, `one-task-at-a-time.typ`, `trace-commits-to-tasks.typ`, new.
- `www/utils/site.typ`: the "Problems" NAV group, first.
- `www/content/estimates.typ`, `hooks.typ`, `commit-rules.typ`: link to the matching problem page instead of retelling it.


== Out of scope

- Problems beyond the five: add a page when an example exists for it.


== Dogfood

Each problem page describes a rule this repo runs on itself; the page links to `.mindtape/config.toml` as a second real example.

