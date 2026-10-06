#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "List the CLI command pages in the sidebar under CLI reference",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a reader sees every command from any page and opens one in a click],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 6),
)

== Problem

+ Open any docs page and read the sidebar's Reference group.
+ 🔴 It shows one "CLI reference" entry. None of the 21 command pages under `/cli/` appear, so a reader reaches `mt link` only through the index or search.


== TO-BE

- 🟢 Under "CLI reference", the sidebar lists every command page in the index's order, one `mt <command>` link each.
- 🟢 The list is a native `<details>` element, so it needs no JavaScript. It is open on `/cli/` and every command page, and closed elsewhere until the reader opens it.
- 🟢 The current command page is highlighted like any other active page.
- 🟢 The list is built from the same command data the index renders, so a new command appears with no edit to `NAV`.


== Source of truth

- `NAV` in `www/utils/site.typ` and `www/generated/cli.json`: the routes and their order.


== Changes

- `www/templates/layout.typ`: render the nested list under the CLI entry.
- `www/utils/site.typ`, or `www/utils/cli-ref.typ`: expose the command routes in index order.
- `www/assets/styles/components.css`: style the nested list and its disclosure marker in both themes.


== Out of scope

- Nesting for any other sidebar entry.
- The `llms.txt` layout, which already lists command pages under the CLI entry.


== Dogfood

Not applicable: the sidebar serves readers of the site.

