#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Build an api-service example around the five problems MindTape solves",
  kind: "dev",
  priority: ice(
    impact: 9,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("docs", "www"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[the example config is the config a software team copies],
      typst-over-markdown: helps[the first task an engineer sees is one that Markdown plus YAML cannot enforce],
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
    "accept-delete-group-and-hyphenated-verbs-in-the-imperative-title-rule.typ",
  )
    + parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Read `examples/`.
+ 🔴 The project that feeds the docs captures and the skill evals is `learn-piano`, a piano practice backlog. MindTape is aimed at software engineers, and the piano tasks show none of the problems they would adopt it for.
+ Those problems are spread over this repo's own `.mindtape/config.toml` and single-subject fixtures (`examples/estimates`, `examples/links`). No project shows them together, so no page can quote one.
+ The five that matter most to an engineer: a size cap forces a big task to be split; `mt add` refuses a task without the required sections; `mt done` runs the project's tests; one task is in progress at a time, in dependency order; every commit names its task.


== TO-BE

- 🟢 `examples/api-service` holds a small backend team's config and three tasks: the showcase task, the task it depends on, and one task in progress.
- 🟢 The config turns on all five solvers in under 30 lines: `extends = ["mindtape-tasks"]`, a `src-files` estimate with a cap rule, a one-in-progress rule, `unblocked` statuses, and an `[[on.flip]]` gate running `scripts/test`.
- 🟢 The showcase task, about throttling failed logins, uses every solver in 25 lines or fewer.
- 🟢 `mt check` passes on the project.


== Changes

- `examples/api-service/.mindtape/config.toml`, new: the config, with `docs:slice` markers per solver.
- `examples/api-service/scripts/test`, new: a stub that prints a failing test name and exits 1, so the gate refusal is the same on every run.
- `examples/api-service/tasks/*.typ`, new: the three tasks, with `docs:slice` markers.
- `examples/api-service/.mindtape/templates/task.typ`, new: an `mt add` template with a slot per required section. The built-in template has no sections, so every `mt add` under the pack would be refused.
- `Justfile`: the config in `TOML_FMT_PATHS`, because `just check-gate-drift` fails on a TOML file the list leaves out.
- `Justfile` and `.mmz/config.yaml`: the tasks and the template in `TYPST_FMT_PATHS` and the `typ-fmt` scope, so `just fmt-check` keeps them formatted.


== Rollback

Delete `examples/api-service/`.


== Out of scope

- Captures and `check-examples` wiring: the next task.
- Removing `learn-piano`: done after its consumers move.


== Dogfood

The config mirrors this repo's own rules (src-files caps, one task in progress, the done gate), so a change to either shows up as a difference between them.

