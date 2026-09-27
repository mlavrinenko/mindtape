#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move open tasks and templates to a src-files estimate",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt",),
  estimate: (files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent estimates the one number that predicts design spread],
      enforced-practice: helps[the drift warning fires on source growth, not on the docs every task touches],
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
  status: proposed(2026, 9, 27),
)

== Problem

+ Four tasks closed in one session each declared `estimate: (files: N)` and each got `warning: working tree touches N file(s), over the M declared by …`.
+ 🔴 The count includes docs pages, tests and `CHANGELOG.md`, which every task here touches by process, so the warning fired every time and carried no signal. The one real miss, a source file the plan left out, was buried in it.


== TO-BE

- 🟢 This repo estimates `src-files`: non-test source a task plans to touch. `.mindtape/config.toml` sets `units = ["src-files", "files"]`, `files` kept only so closed tasks stay valid untouched.
- 🟢 `[tasks.estimate.drift]` reads `unit = "src-files"`, and its `exclude` list drops docs, tests, `CHANGELOG.md` and `truth/tasks/**`, so the warning counts source only.
- 🟢 The caps key on `src-files`; the `files = 1` minimum goes, since a docs-only task touches no source.
- 🟢 Every template's `estimate` default and prompt say `src-files`; open tasks are recounted as source only.


== Changes

- `.mindtape/config.toml`: units, drift unit and `exclude`, caps.
- `.mindtape/templates/*.typ`: the `estimate` defaults and prompts.
- Open tasks under `truth/tasks/` (`mt ls '!terminal && has(estimate)'`): `files:` to `src-files:`, recounted.


== Rollback

Revert the commit; closed tasks are never touched, so nothing else moves.


== Out of scope

- Rewriting closed tasks: no rule reads a closed task's estimate.
- Filters reading `estimate.src-files`: the lexer learns kebab-case in `truth/tasks/read-estimate-units-and-kebab-case-names-in-filters.typ`; nothing filters on the unit before then.
- Replacing the drift and bounds tables with rules: `truth/tasks/retire-the-drift-check-and-estimate-bounds-in-favour-of-rules.typ`.


== Dogfood

The next task filed here estimates `src-files`, and its drift warning counts source only.

