#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Point mt --help, the quickstart and the sidebar at mt @name",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("docs", "www", "cli"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[someone with many repos finds `mt @name` from the first help screen or page they open],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the `/cli/` index shows the help's own closing lines from `cli.json`],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 17),
  links: parent("keep-the-user-docs-generated-or-gated-not-restated.typ"),
)

== Problem

+ Run `mt --help`, then open the home page, `/quickstart/` and `/cli/`, looking for a way to read several projects.
+ 🔴 `mt --help` ends on "Run `mt flip --list` to see them all.", and none of the four names `@name`.
+ 🔴 The page sits under the sidebar's "Integrate", the group `www/utils/site.typ` keeps for "wiring `mt` into a repo, a tool or another tracker".


== TO-BE

- 🟢 `mt --help` ends with a line on `@name`, and `/cli/` renders the same lines from `cli.json`:
```text
Every status is a command too, like `mt wip <task>` and `mt done <task>`.
Run `mt flip --list` to see them all.
Put `@name` first, as in `mt @work ls`, to run in a project or set `mt at` lists.
```
- 🟢 The quickstart ends on a line linking `/read-many-projects/`.
- 🟢 The page sits in "Start", which holds 4 of the 6 a group may. A reader with many repos asks this on day one.


== Source of truth

- `crates/mindtape-cli/src/cli/mod.rs`: the root `after_help`.
- `www/utils/site.typ`: `NAV` and its six-page rule.


== Changes

- `crates/mindtape-cli/src/cli/tree_json_tests.rs`: the red test first: the dump carries the root `after_help`, `@name` line included.
- `crates/mindtape-cli/src/cli/mod.rs`: the `@name` line.
- `crates/mindtape-cli/src/cli/tree_json.rs`: export the root `after_help`.
- `www/content/cli.typ`: render it.
- `www/content/quickstart.typ`: the closing line.
- `www/utils/site.typ`: move the page to "Start". Recipes holds seven pages against the six-page rule in the same comment, so split it or restate the rule.


== Out of scope

- The page's content, which its sibling tasks rewrite.


== Dogfood

The owner runs `mt --help` in this repo, and the line points at the `mt @home` board the owner reads daily.

