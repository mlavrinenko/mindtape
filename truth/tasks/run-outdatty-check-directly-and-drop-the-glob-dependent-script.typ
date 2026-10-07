#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Run outdatty check directly and drop the glob-dependent script",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: (),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the native check also fails a listed dependent that has no lock entry],
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
  status: done(2026, 10, 7),
)

== Problem

+ Read `.just/scripts/outdatty-check.sh`.
+ 🔴 It re-implements what outdatty 0.6.0 does itself, and only for glob dependents, so nine listed `examples/task-files` dependents had no lock entry unnoticed.

== TO-BE

- 🟢 `just outdatty-check` runs `outdatty check --format paths` and fails on any dependent with no lock entry, listed or glob.

== Changes

- `.just/scripts/outdatty-check.sh`, `.mmz/config.yaml`, `outdatty.yaml`: the script, its mmz scope and its group entry go.
- `Justfile`, `docs/src/contributing.typ`: the recipe calls `outdatty check` and the doc sentence is reworded.
- `flake.lock`, `outdatty.lock`: outdatty 0.6.0, and the nine task-files dependents recorded.

== Rollback

- Revert the commit.

== Out of scope

- The qahq flake, bumped in its own repo.

== Dogfood

Not applicable, this is the harness.
