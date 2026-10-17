#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Document reading many projects with mt @name",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (src-files: 0),
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
      docs-match-code: helps[every example on the page is a captured run over a checked fixture],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17),
  links: parent("show-every-tracked-project-from-one-mt-name.typ")
    + depends-on(
      "list-tasks-from-every-project-of-a-set-in-one-grouped-table.typ",
      "check-every-project-of-a-set-with-mt-name-check.typ",
      "show-one-row-per-project-with-mt-board-and-a-bare-mt-name.typ",
    ),
)

== Problem

+ Open the docs site and look for a way to see several projects at once.
+ 🔴 Only `www/content/agents.typ` covers `@name`, as one project per name. Nothing shows a set, the board, or which commands run on a set.


== TO-BE

- 🟢 A problem-led page shows the board, a set's `ls` and `check`, the `at.toml` shape and the `forks` modes, each example a captured run over a fixture.
- 🟢 The skill says which commands take a set and that a write needs one project.
- 🟢 `www/content/agents.typ` links the page from its worktree section.


== Source of truth

- `crates/mindtape/src/at/`: the `at.toml` keys, the shipped board and how `@x` resolves.
- `crates/mindtape-cli/src/util.rs`: `open_set`, whose callers are the commands that take a set.
- Captured runs over `examples/across-projects/`.


== Changes

- `examples/across-projects/`: two small projects and the `at.toml` the captures read through `MINDTAPE_CONFIG_HOME`.
- `www/content/read-many-projects.typ`: the new page.
- `www/content/agents.typ`: link the page.
- `docs/src/skill.typ`: sets and writes, then regenerate `skills/mindtape/SKILL.md`.
- `outdatty.yaml`, `outdatty.lock`: the page's sources.


== Out of scope

- The `mt at` and `mt board` reference pages, which their command tasks generate.
- README, which names no `@x`.


== Dogfood

This repo's docs gate captures the page's runs, so a change to sets fails until the page follows.

