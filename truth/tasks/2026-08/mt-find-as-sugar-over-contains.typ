#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "mt find as sugar over contains",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli", "docs"),
  estimate: (files: 8),
  status: done(2026, 8, 8)[landed 2026-08-08, just check green on merged HEAD],
  links: depends-on("unify-search-and-contains-into-one-function.typ")[
    `find` is sugar over the unified `contains`, so it lands after it
  ],
)

== Summary

Searching by hand costs more keystrokes than the search is worth:
`mt ls 'contains("tombstone")'` is a function call inside a quoted expression
inside a shell. A verb removes the ceremony: `mt find tombstone`.

The value is only in the typing, so `find` must stay pure sugar. The moment it
grows options of its own, it becomes a second query engine that drifts from the
filter language. That is the failure `fields.rs` exists to prevent after
`filter.rs` and `sort.rs` disagreed about which fields exist.

== Scope

- `mt find TERM…` reduces to a filter expression and hands it to the same code
  path `mt ls` uses. No second matcher, no separate field vocabulary.
- Multiple terms conjoin: `mt find login bug` becomes
  `contains("login") && contains("bug")`. A search box behaves that way, and the
  existing language can express it. The sugar needs nothing new from the
  language.
- Every `mt ls` flag forwards verbatim: `--format`, `--json`, `--sort`,
  `--columns`, `-n/--limit`, `-i/--interactive`, `--color`, `--hyperlinks`,
  `-w/--worktree`. Not one flag `ls` lacks. Field targeting stays in the filter
  language (`mt ls 'contains("bug", title)'`), with no `--in` flag.
- The reduced expression is visible, not hidden. It lands in the `applied` facet
  the same way a manual filter does, so `mt find bug --json` reports
  `contains("bug")` as the applied filter and the table footer names it.
- Deliberately out of scope: pluggable external search backends. Measured on
  `examples/large` (2000 tasks), a filtered `mt ls` costs about 90 ms end to end
  while `rg -l` over the same corpus costs 11 ms. The 90 ms is loading and
  evaluating tasks, not matching text, and a backend still needs those tasks
  loaded to render a row or resolve an id. The only way a backend wins is a
  persistent index, which contradicts the daemon-free, no-global-index premise
  the project builds on. Revisit only if a corpus appears where the match, not
  the load, is the cost.

== Notes

`crates/mindtape-cli/src/cli.rs` is an outdatty source for both the `cli-docs`
and `cli-name` groups. A new subcommand drags a wide dependent set. Those
dependents are `www/content/{cli,filters,config,quickstart}.typ`,
`www/utils/cli-notes.typ`, `skills/mindtape/SKILL.md`, `README.md`,
`www/content/index.typ`, `CONTRIBUTING.md` and `AGENTS.md`. Reconcile them, then
run `just outdatty-update`.
