#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Capture the api-service example's problem-solver runs for the docs",
  kind: "dev",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs", "www", "tooling"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: helps[each solver is shown by `mt` refusing a bad task or flip, which a YAML file cannot do],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[each solver's demo is a real run on every build],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
  links: depends-on(
    "build-an-api-service-example-around-the-five-problems-mindtape-solves.typ",
    "print-each-captures-command-from-the-string-that-ran-it.typ",
  )
    + parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Look for a captured `mt` refusal in `www/generated/`.
+ 🔴 There is none for a size cap at `mt add`, a missing section, a failing done gate, a second task in progress, or a commit without `Refs:`. These refusals are what a reader should see first, and today only prose describes them.


== TO-BE

- 🟢 `www/generate.sh` copies `examples/api-service` into a scratch git repo and captures, with the capture helper: `mt add` refused by the size cap and by missing sections; `mt done` refused by the test gate; a second `mt wip` refused; `mt ls blocked --columns title,why`; `mt check --commit-msg` refusing a message without `Refs:` and accepting one with it; `mt ls 'touches(...)'` after a commit.
- 🟢 `just check-examples` validates `examples/api-service`.


== Changes

- `www/generate.sh`: the scratch repo and the captures. A capture whose command unexpectedly succeeds fails the build, because a refusal demo that passes shows nothing.
- `examples/api-service/tasks/*.typ`: any task the captures need.
- `Justfile`: add the project to `check-examples`.
- `.mmz/config.yaml`: the project in the `examples` scope.
- `outdatty.yaml`: couple the project to the pages that quote it.


== Rollback

Revert the commit; the generator stops capturing the project.


== Out of scope

- Pages that show the captures: the README and problem-page tasks.


== Dogfood

`just check` regenerates the captures, so a change to `mt`'s refusal messages shows up in this repo's diff.

