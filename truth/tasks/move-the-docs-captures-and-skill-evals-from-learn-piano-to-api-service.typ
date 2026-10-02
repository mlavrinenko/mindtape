#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move the docs captures and skill evals from learn-piano to api-service",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (src-files: 8),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[every page captures one project an engineer recognizes],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
  links: depends-on(
    "capture-the-api-service-examples-problem-solver-runs-for-the-docs.typ",
  )
    + parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Run `rg -l learn-piano www docs/src skills .just Justfile`.
+ 🔴 The CLI, quickstart, filters, templates and import-export captures, the skill's triage evals and the contributor docs all read `examples/learn-piano`. Its tasks practise scales and Clair de Lune, so every transcript on the site shows a piano backlog to an audience of software engineers.


== TO-BE

- 🟢 Every capture that read `learn-piano` reads `api-service`, and the pages' prose describes the new output.
- 🟢 The skill evals that used `learn-piano` use `api-service`, with the same checks.
- 🟢 Nothing outside `examples/learn-piano/` names `learn-piano`.


== Changes

- `www/generate.sh`, `www/generate-examples.sh`: point the captures at `api-service`.
- `examples/api-service/`: the tasks, views, templates and dict the captures need.
- `skills/mindtape/evals/evals.json`: the triage evals on `api-service`.
- `www/content/*.typ`, `docs/src/**`: prose that names the project or its tasks.


== Rollback

Revert the commit; captures go back to `learn-piano`.


== Out of scope

- Deleting `learn-piano`: the next task, once nothing reads it.


== Dogfood

The skill evals this repo runs use the new project.

