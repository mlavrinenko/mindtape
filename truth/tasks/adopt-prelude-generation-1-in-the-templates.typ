#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Adopt prelude generation 1 in the templates",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt", "tooling", "stateless"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[a new task opens with one project prelude import],
        cheap-adoption: helps[a project rewires 16 templates instead of every task],
      ),
      ..(
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (sound-codebase: helps[each prelude generation is one frozen module]),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 8),
  links: related(
    "give-each-project-one-prelude-module.typ",
  )[implements the decision],
)

== AS-IS

+ Open `.mindtape/templates/task.typ`.
+ 🔴 It carries the library version and the helper path in two lines, so every task filed from it records the current library pin.
```
$ head -2 .mindtape/templates/task.typ
#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *
```


== TO-BE

- 🟢 `truth/prelude/1/lib.typ` is the first frozen project prelude.
- 🟢 Every template imports it in one root-absolute line, so `mt add` writes tasks at the current generation.
```typ
#import "/truth/prelude/1/lib.typ": *
```


== Changes

- `truth/prelude/1/lib.typ`: pins `@local/mindtape:0.4.0` and exports the project's task-facing helpers.
- `.mindtape/templates/task.typ`: import generation 1.
- `.mindtape/templates/decision.typ`: import generation 1.
- `.mindtape/templates/dev.typ`: import generation 1.
- `.mindtape/templates/dev-stateful.typ`: import generation 1.
- `.mindtape/templates/docs.typ`: import generation 1.
- `.mindtape/templates/feat.typ`: import generation 1.
- `.mindtape/templates/feat-stateful.typ`: import generation 1.
- `.mindtape/templates/finding.typ`: import generation 1.
- `.mindtape/templates/fix.typ`: import generation 1.
- `.mindtape/templates/fix-stateful.typ`: import generation 1.
- `.mindtape/templates/sweep-dry.typ`: import generation 1.
- `.mindtape/templates/sweep-mutants.typ`: import generation 1.
- `.mindtape/templates/sweep-repeats.typ`: import generation 1.
- `.mindtape/templates/sweep-size.typ`: import generation 1.
- `.mindtape/templates/sweep-undertested.typ`: import generation 1.
- `.mindtape/templates/tweak.typ`: import generation 1.


== Out of scope

- The old task files. Each keeps the literal pin it was written against.
- Rust, schema, config and `mt fix`; a prelude is a project convention, not an `mt` concept.


== Dogfood

The next task this repo files imports `truth/prelude/1/lib.typ`, and the templates that file it do too.

