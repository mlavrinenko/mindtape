#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the CLI, filters, priority, named-sets, hooks and estimates pages plainly",
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
      docs-match-code: helps[each filter and hook example runs in the project it names],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
  links: depends-on(
    "write-plain-prose-rules-for-the-docs-and-check-for-history-and-shouting.typ",
    "refuse-hand-typed-code-blocks-in-doc-sources.typ",
    "split-the-config-page-into-config-statuses-and-rules-pages.typ",
    "print-each-captures-command-from-the-string-that-ran-it.typ",
  )
    + parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Run the pages' examples and read their claims.
+ 🔴 `filters.typ:141` and `:157` use `in_dict("verbs", …)`. In a fresh project it prints `unknown dict in filter: verbs`: the `verbs` dict comes from the mindtape-tasks pack.
+ 🔴 `filters.typ:186` says `into()` works only in `[[on.flip]]`; task-scope `[[commits.rules]]` accepts it too (`commit_task_expr.rs:127`).
+ 🔴 `cli.typ:282` links commit rules to /config/; `cli.typ:207` counts "the last four" but includes the `true`/`false` row.
+ `hooks.typ:34` and `:81` describe this repo's `mmz --is-fresh` setup as if every reader had it. The WIP-cap TOML is identical at `filters.typ:279` and `config.typ:583`.


== TO-BE

- 🟢 Every claim matches the code, and every example runs in the project it names.
- 🟢 `[[ls.rules]]` lives on the page the config split chose; these pages link to it.
- 🟢 Hooks examples use a command any project has.
- 🟢 Blocks are fixture slices or captures; the prose follows the plain-prose rules.


== Source of truth

- `crates/mindtape/src/filter*` and `commit_task_expr.rs`: functions and where they apply.
- `crates/mindtape/src/packs/mindtape-tasks.toml`: pack dicts and rules.
- `mt <cmd> --help`.


== Changes

- `www/content/{cli,filters,priority,named-sets,hooks,estimates}.typ`: rewrite whole.
- `www/utils/cli-notes.typ`, `www/utils/framework-notes.typ`: match the CLI and priority pages; their blocks become slices or captures.
- `examples/*`: slices for the filter, hook and estimate blocks.
- `.just/scripts/check-doc-blocks.sh`: drop these pages from the list.


== Out of scope

- The filter language itself: docs only.


== Dogfood

This repo's own `.mindtape/config.toml` uses these filters and hooks; slices from it keep the pages honest.

