#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "List tasks from every project of a set in one grouped table",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("cli",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[tasks from every project list in one run, read straight from their files],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 17),
  links: parent("show-every-tracked-project-from-one-mt-name.typ")
    + depends-on(
      "resolve-mt-name-through-at-toml-and-rename-mt-worktrees-to-mt-at.typ",
      "carry-the-project-root-and-hyperlink-choice-in-renderctx.typ",
      "load-every-project-of-a-set-in-parallel.typ",
    ),
)

== Problem

+ Define `[at.home]` over `~/projects/home/*` and run `mt @home ls ready`.
+ 🔴 No command lists tasks from two projects. Today it prints `error: unknown worktree: '@home'`, and a shell loop prints one table per project with no project name on a row.


== TO-BE

- 🟢 `mt @home ls` and `mt @home find` print one table: a PROJECT column first, then rows grouped by project in set order.
- 🟢 Each project applies its own config: its default listing, its `@view`, its sort, and `--sort` within it. A project lacking the view is an error naming it, and the rest still list.
- 🟢 `-n` and `--skip` count per project, and the footer reads `your filter · 9 of 61 tasks in 27 projects`.
- 🟢 The columns are `--columns`, else the union of what each project's listing shows.
- 🟢 `--json` prints one object per project with its `applied`, `tasks` and error, and CSV gains a `project` column.
- 🟢 A failed project exits 1 after the rest print.


== Changes

- `crates/mindtape-cli/tests/it/ls_set.rs`: the red tests first, over two fixture projects and an `at.toml` under `MINDTAPE_CONFIG_HOME`: grouping, `-n` per project, a missing view, a broken member.
- `crates/mindtape-cli/src/cmd/ls/set.rs`: resolve each member's listing from `open_set`, cut it per project, and join the groups.
- `crates/mindtape-cli/src/cmd/ls/mod.rs`: send a set to `set.rs`, and the set footer.
- `crates/mindtape-cli/src/cmd/find.rs`: send a set the same way.
- `crates/mindtape-cli/src/render/columns.rs`: the `project` column, shown only for a set.
- `crates/mindtape-cli/src/render/output.rs`: the set JSON shape and the CSV column.


== Alternatives

- One sort merged across projects: priority scales differ per project, so it would compare ICE scores with plain numbers.
- One table per project: the header repeats, and the output no longer reads as one list.
- `-n` as a total: `mt @home ls -n 10` would show mostly the first project.


== Out of scope

- `--interactive` on a set: it refuses one until someone asks for it.
- Writes from the listed rows: a write still needs one project.


== Dogfood

The owner picks the next task across `~/projects/home`, this repo included, with `mt @home ls ready`.

