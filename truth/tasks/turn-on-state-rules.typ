#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Turn on the statefulness and example rules after the migration",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 2,
  tags: ("mt", "stateless"),
  estimate: (src-files: 1),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[the switch is explicit and the rules turn on together],
      ),
      "readable-tasks",
      "implementable-tasks",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (:),
      "sound-codebase",
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 2),
  links: depends-on("task-shape-docs.typ") + depends-on("as-is-sweep.typ"),
)

== AS-IS

+ Run `mt tags --for add` and read the statefulness and example rules.
+ 🔴 Both are parameter-gated off, so a new task can still omit the statefulness tag and the example fence.
+ 🔴 The templates and fixtures pass only because the rules are off.

== TO-BE

- 🟢 The project's `extends` sets `statefulness` and `examples` true.
- 🟢 The templates, fixtures and filed tasks pass with the rules on.
- 🟢 `mt tags --for add` lists the statefulness and example rules.

```toml
extends = [{ pack = "mindtape:tasks", with = { statefulness = true, examples = true } }]
```

== Changes

- `.mindtape/config.toml`: the two `with` values and the tags the rules require.

== Out of scope

- The pack rules in `task-section-rules`, the templates in `task-templates` and the docs in `task-shape-docs`.

== Dogfood

This repo's own tasks are the first to carry the statefulness tags and example fences.
