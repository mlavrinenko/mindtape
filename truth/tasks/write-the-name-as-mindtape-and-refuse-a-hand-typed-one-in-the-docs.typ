#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write the name as Mindtape and refuse a hand-typed one in the docs",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs", "tooling"),
  estimate: (src-files: 25),
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
      docs-match-code: helps[a doc page renders the name from one constant, and a gate refuses a typed one],
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    12,
  )[the docs render the name from the site title, and check-doc-prose refuses a typed one],
)

== Problem

+ Run `rg -c MindTape www docs/src crates`.
+ 🔴 The name is typed out in prose, help text and the config schema, about 120 times, as `MindTape`.


== TO-BE

- 🟢 Every doc source renders the name as `#mindtape`, from one constant.
- 🟢 `just check-doc-prose` fails on `Mindtape` or `MindTape` typed in doc prose.
- 🟢 `mt --help`, the config schema and the crate descriptions say Mindtape.


== Source of truth

- The owner's docs review: one spelling, Mindtape, from one constant, checked by a gate.


== Changes

- `www/utils/site.typ`: `mindtape`, the constant, and `SITE.title` read from it.
- `.just/scripts/check-doc-prose.awk`: refuse the typed name in prose.
- `crates/mindtape-cli/src/cli/mod.rs`, `crates/mindtape/src/config/schema.rs`, the crate manifests and `schema/config.schema.json`: the spelling.
- The comments that name the product, in code and config.


== Out of scope

- Task files under `truth/tasks/`: a closed task is history.
- Released `CHANGELOG.md` entries, for the same reason.


== Dogfood

This repo's own docs and `mt --help` are what the change rewrites.

