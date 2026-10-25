#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Capture mt packs output in the docs from a local git fixture",
  kind: "docs",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 2),
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
      docs-match-code: helps[the Packs page shows what `mt packs install` and `update` really print],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on("print-what-a-pack-update-changes.typ"),
)

== Problem

+ Read the "Packs from a git source" section of `www/content/packs.typ`.
+ 🔴 Its `mt packs` output is typed by hand: the docs build has no git source to fetch from, so the text can drift from what the commands print.


== TO-BE

- 🟢 The docs build makes a bare git repository under its scratch directory, with two tagged commits of one pack, and captures `mt packs install` and `mt packs update` against it.
- 🟢 The page renders those captures with `terminal(...)`, like every other example.


== Source of truth

`crates/mindtape-cli/src/cmd/packs/mod.rs`, whose output the captures record.


== Changes

- `www/generate-packs.sh`: build the fixture repository and capture both commands, with `MINDTAPE_CACHE_HOME` pointing inside the scratch directory.
- `www/generate.sh`: export the scratch cache directory before it runs the generator.
- `www/content/packs.typ`: render the captures.


== Out of scope

- A fixture hosted anywhere but the build's scratch directory: the docs build stays offline.


== Dogfood

This repo's docs build captures the two commands on every `just docs`.

