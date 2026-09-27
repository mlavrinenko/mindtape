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
  estimate: (src-files: 11),
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
  status: wip(2026, 9, 28),
  links: depends-on("check-estimate-units-on-open-tasks-only.typ"),
)

== Problem

+ Four tasks closed in one session each declared `estimate: (files: N)` and each got `warning: working tree touches N file(s), over the M declared by …`.
+ 🔴 The count includes docs pages, tests and `CHANGELOG.md`, which every task here touches by process, so the warning fired every time and carried no signal. The one real miss, source files the plan left out, was buried in it.


== TO-BE

- 🟢 Open tasks and templates estimate `src-files`: non-test source a task plans to touch. `files` is retired with a one-line edit, and the 362 closed tasks keep it untouched:

```toml
[tasks.estimate]
units    = ["src-files"]
required = "!terminal"
```

- 🟢 Drift reads `unit = "src-files"` and `exclude`s docs, tests, `CHANGELOG.md` and `truth/tasks/**`, so the warning counts source only.
- 🟢 The caps key on `src-files`, with no minimum: a docs-only task touches no source.


== Changes

- `.mindtape/config.toml`: `units`, drift unit and `exclude`, caps.
- `.mindtape/templates/*.typ`: the `estimate` defaults and prompts.
- Open tasks under `truth/tasks/` (`mt ls '!terminal && has(estimate)'`): `files:` to `src-files:`, recounted as source only.


== Rollback

Revert the commit; closed tasks are never touched, so nothing else moves.


== Out of scope

- Rewriting closed tasks: no rule reads a closed task's estimate.
- Replacing the drift and bounds tables with rules: `truth/tasks/retire-the-drift-check-and-estimate-bounds-in-favour-of-rules.typ`.


== Dogfood

The next task filed here estimates `src-files`, and its drift warning counts source only.
