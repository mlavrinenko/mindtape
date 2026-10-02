#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Trim SKILL.md and AGENTS.md to what an agent acts on",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent reads each rule once and spends less context on the skill],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
  links: depends-on(
    "write-plain-prose-rules-for-the-docs-and-check-for-history-and-shouting.typ",
    "lint-the-rendered-markdown-for-typlite-traps-and-fix-the-current-hits.typ",
  )
    + parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Read `skills/mindtape/SKILL.md`.
+ 🔴 `:14` and `:77` both state the hand-edit exception. `:39` restates `mt set --help` just before `:41` says flags live in `--help`. `:53` repeats the `k=v`/`k+=v`/`k@path` syntax. `:47` is a 200-word list of filter fields and helpers.
+ 🔴 `:47` "tasks the trailers of the commits that changed a file name" does not parse, and `:14` "and writes validate before landing" lacks a subject.
+ Read `AGENTS.md`. 🔴 `:43-51` list crate versions and explain how the list is generated; `:11` gives the repo URL to an agent that is already in the repo.


== TO-BE

- 🟢 SKILL.md states each rule once and keeps what an agent acts on. The filter field and helper list moves to a reference file the skill links.
- 🟢 AGENTS.md says what MindTape is and where things live, with no versions or build notes.
- 🟢 Both follow the plain-prose rules.


== Source of truth

- `mt --help` and `mt <cmd> --help`.
- `crates/mindtape/src/filter*`: the field and helper list.


== Changes

- `docs/src/skill.typ`: rewrite whole.
- `skills/mindtape/references/filters.md`, new: the field and helper list, with `touches()` worded as "tasks named in the `Refs:` trailers of commits that changed that file".
- `docs/src/agents.typ`: rewrite whole.


== Out of scope

- `skills/mindtape/evals/`: the skill teaches the same things, so the evals stand.


== Dogfood

Agents in this repo load the trimmed skill and AGENTS.md every session.

