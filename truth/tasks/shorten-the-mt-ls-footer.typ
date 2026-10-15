#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Shorten the mt ls footer",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("cli", "www"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: helps[the line under a listing names the view and the sort rule, nothing more],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the ls page quotes the footer the code prints],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 16),
)

== Problem

+ Run `mt ls` in a project whose matching `[[ls.rules]]` entry imports the `inbox` view and sets its own sort.
+ 🔴 The footer reads `filter and columns from inbox view · sort from release rule · 3 of 12 tasks`.
+ Run `mt ls 'ready'` where that rule sets only a filter.
+ 🔴 The footer reads `your filter · release rule overridden · 1 of 4 tasks`.


== TO-BE

- 🟢 The footer names the view, then a sort rule as `<name> sort`, then the count, as in `inbox view · release sort · 3 of 12 tasks`.
- 🟢 It no longer says which facet came from where, and no longer says a rule was overridden. `mt ls --json` keeps `superseded`.


== Changes

- `crates/mindtape-cli/src/cmd/ls/rules_applied_ls_rules_tests_applied.rs`: the red tests first, the new forms.
- `crates/mindtape-cli/src/cmd/ls/rules_applied.rs`: render the shorter footer.
- `www/content/cli/ls.typ`: describe the shorter footer.


== Alternatives

- Keep the facet names and only drop the overridden note: still reads like config debugging.


== Out of scope

- The `applied` object in `mt ls --json`. It keeps every facet and `superseded`.


== Dogfood

This repo's `mt ls` prints the shorter footer.
