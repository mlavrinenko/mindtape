#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add mt mv to rename a task and rewrite its inbound links",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("cli", "mt"),
  estimate: (files: 7),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a rename goes through the resolver rather than a find-and-replace],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[identity stays the filename, and changing it stays a supported move],
    ),
    process: (
      sound-codebase: helps[the link-rewriting path gets a second caller and a second test],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 9, 22),
)

== Problem

+ Identity is the filename stem, so a task whose title turned out wrong
  carries a stem that names the wrong thing forever.
+ 🔴 Renaming it means `git mv` plus a hand edit of every inbound
  `links:` target, with nothing checking the result until `mt check` reports
  a dangling edge. Miss one and the graph quietly loses an edge.
+ `mt rm` already walks that graph and rewrites inbound links, in
  `crates/mindtape-cli/src/rm_unlink.rs` and `rm_gitless.rs`. It points them
  at nothing; a rename would point them at the new stem.


== TO-BE

- 🟢 `mt mv <task> <new-stem>` moves the file and rewrites every inbound
  `links:` target to the new stem, in one validated write.
- 🟢 `--dry-run` prints the plan: the move, and every file whose edges change.
- 🟢 A stem already taken is refused before anything moves.
- 🟢 The move uses git when the tree is a repository and a plain rename when
  it is not, the way `mt rm` already splits.
- 🟢 `mt check` is clean straight after, with no dangling edge anywhere.


== Changes

- `crates/mindtape-cli/src/mv.rs` — new `Args`/`run` pair. Extract the
  inbound-edge walk `rm` already owns rather than copying it; `check-dry`
  fails on the copy anyway. Red test first.
- `crates/mindtape-cli/src/rm_unlink.rs` — generalise from `remove the edge`
  to `retarget the edge`, with removal as the null target.
- `crates/mindtape-cli/src/cli.rs` and `command_class.rs` — wire it, and
  declare it a write command.
- `crates/mindtape-cli/tests/` — end-to-end arms for the git and gitless
  paths, a refused collision, and `--dry-run` leaving the tree untouched.
- `www/utils/cli-notes.typ`, `www/content/cli.typ` and
  `skills/mindtape/SKILL.md` — the new verb.


== Alternatives

- A `--rename` flag on `mt set title=...`. Rejected: a title change and a
  stem change are different decisions, and coupling them would rename a file
  every time someone fixes a typo.
- Leaving it to `git mv` plus `mt check`. Rejected because the check reports
  the damage after the fact instead of preventing it.


== Out of scope

- Changing `title:` as a side effect. The stem and the title are independent
  by design, and `mt set` owns the title.
- Moving a task between roots or folders, which changes its derived tags and
  deserves its own decision.


== Dogfood

This backlog has stems written before their task settled. The first one
renamed with `mt mv` proves the inbound rewrite, because `mt check` here
resolves several hundred edges.

