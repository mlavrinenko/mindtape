#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Correct the false task-model claims on the docs site",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 1,
  estimate: (src-files: 5),
  tags: ("found", "www"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a reader following the priority and import pages succeeds first time],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[five task-model claims match the code],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 8),
)

== Problem

+ Reviewing the docs before release against the 0.3.0 binary
+ 🔴 #raw("www/content/schema.typ"): five pages state task-model behaviour the code does not have

- `schema.typ:33,94` vs `cmd/import/mod.rs:134`: named from `id`; `id:` is authored too.
- `dsl.typ:50`: a declared `x-team` fails `'core' is not an allowed value`.
- `priority.typ:105`: `framework 'mine' used but no priority framework is configured`.
- `recurring.typ:87`: the key is `default-anchor`.
- `import-export.typ:59`: `recurring`, `waiting` records pass `taskwarrior.jq` unmapped and get skipped.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("www/content/schema.typ").

== TO-BE

- 🟢 Each claim above matches the code, verified by running it, and each page states its fact once and links where another page owns it.
- 🟢 The import naming rule lives on `import-export.typ`, and `schema.typ` links to it.


== Changes

- `www/content/schema.typ`: `id` and import naming.
- `www/content/dsl.typ`: extension fields, linking to `concepts.typ`.
- `www/content/priority.typ`: the missing config line, in the checked example if one exists.
- `www/content/recurring.typ`: the key name.
- `www/content/import-export.typ`: say what happens to other Taskwarrior statuses, or map them in `contrib/jq/taskwarrior.jq` if a mapping is obvious.


== Rollback

- Revert the commit.


== Out of scope

- Changing import's naming rule.


== Dogfood

Not applicable: these pages describe features this repo's backlog does not exercise.

