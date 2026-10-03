#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the rule-packs, commit-rules, schema, import-export and internals pages plainly",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("www", "docs"),
  estimate: (src-files: 3),
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
      docs-match-code: helps[the import and export recipes round-trip a real project],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
  links: depends-on(
    "write-plain-prose-rules-for-the-docs-and-check-for-history-and-shouting.typ",
    "refuse-hand-typed-code-blocks-in-doc-sources.typ",
    "print-each-captures-command-from-the-string-that-ran-it.typ",
  )
    + parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Run the pages' recipes against a fixture.
+ 🔴 `import-export.typ:132` exports with `mt ls --json`, which applies the default view: 9 of learn-piano's 13 tasks come out, and the recipe's `done` branch never runs. The CSV recipe (`:143`) fails on a framework priority. The adapters emit `todo`, which the default `flow` legend lacks.
+ 🔴 `rule-packs.typ:171` names `imperative-title` as the pack's only rule outside the body. The pack also ships `work-names-a-task` and `flip-with-work` (`packs/mindtape-tasks.toml:335`, `:356`), which `commit-rules.typ` presents as rules you write.
+ 🔴 `internals.typ:50` says a cache lives under each root; there is one, `<project>/.mindtape/cache/` (`cache.rs:76`).
+ `schema.typ:18` promises to name the one field that does not round-trip, and never does.


== TO-BE

- 🟢 The export recipes use `mt ls '' --json`, and an import of their output into a fresh project round-trips every task the page says it does; the generator runs them.
- 🟢 Every claim matches the code; the pack's rules are listed once, on /rule-packs/.
- 🟢 Blocks are fixture slices or captures; the prose follows the plain-prose rules.


== Source of truth

- `crates/mindtape/src/canonical.rs`, `cache.rs`: schema and cache.
- `crates/mindtape/src/packs/mindtape-tasks.toml`: pack contents.


== Changes

- `www/content/{rule-packs,commit-rules,schema,import-export,internals}.typ`: rewrite whole. `schema.typ:117`: `note` is null whenever the edge has no note. Name `id` as the field re-minted from the title.
- `www/generate.sh`: run the jq recipes against a fixture and capture the result.
- `.just/scripts/check-doc-blocks.sh`: drop these pages from the list.


== Out of scope

- Changing `mt import` or the canonical schema: docs only.


== Dogfood

The captured round trip runs on every `just check` in this repo.

