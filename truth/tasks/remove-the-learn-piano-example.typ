#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Remove the learn-piano example",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("docs",),
  estimate: (src-files: 4),
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
      docs-match-code: helps[one fewer fixture to keep in step with the code],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
  links: depends-on(
    "move-the-docs-captures-and-skill-evals-from-learn-piano-to-api-service.typ",
  )
    + parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ After the captures move, run `rg -l learn-piano . -g '!examples/learn-piano/**' -g '!truth/**'`.
+ 🔴 Only gate wiring still names it (`Justfile` `check-examples`, `TYPST_FMT_PATHS`, `.mmz/config.yaml`), so the gate keeps validating a fixture nothing reads.


== TO-BE

- 🟢 `examples/learn-piano/` is gone, and no gate, path list or doc names it.


== Changes

- `examples/learn-piano/`: delete. The fixture is test data, so its files are not counted in the estimate.
- `Justfile`: drop it from `check-examples` and `TYPST_FMT_PATHS`.
- `.mmz/config.yaml`: drop it from the `examples` scope.
- `outdatty.yaml`: drop any coupling.
- `.linecop.yaml`: drop any entry.


== Rollback

Revert the commit to restore the directory.


== Out of scope

- The other single-subject fixtures: pages still slice them.


== Dogfood

`just check` gets one example fewer to validate.

