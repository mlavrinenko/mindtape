#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read many projects through one mt @name selector",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 6,
  tags: ("decision", "cli"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[an overview of every project reads their files on each run, with no index or daemon],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    16,
  )[agreed: at.toml with \[at.\<name>\] sets, bare mt \@x runs board, ls grouped by project, forks opt-in, no default board outside a project],
  links: supersedes("worktree-aliases.typ")
    + related(
      "named-views-and-contexts.typ",
    )[a set is picked per command, never remembered as a context],
)

== Problem

+ Keep Mindtape tasks in many repos, such as the 30 under `~/projects/home`.
+ Run `mt @home ls` to see them together.
+ 🔴 It prints `error: unknown worktree: '@home'`, because `@name` and `worktrees.toml` pick one project per run.


== TO-BE

- 🟢 `~/.config/mindtape/at.toml` replaces `worktrees.toml`, and `mt at` replaces `mt worktrees`. Each `[at.<name>]` names a project or a set through `include`, `exclude` and `forks`.
- 🟢 On a set, `ls`, `find`, `check` and `board` run on every project in parallel, each under its own config. Other commands refuse a set.
- 🟢 A bare `mt @x` runs `mt @x board`, one row per project, with columns from `[board]`.
- 🟢 `mt @x ls` groups rows by project, with `-n` per project.
- 🟢 Forks join a set only when its `forks` says so, and a broken project is a warning, not an abort.


== Changes

- `truth/tasks/show-every-tracked-project-from-one-mt-name.typ`: no code here. That milestone's children build the design.


== Alternatives

- A `-p/--projects` flag beside `@x`: two ways to pick projects.
- A separate `overview.toml`: two lists of projects to keep in step.
- A current set that `mt` remembers between runs: the stateful context `named-views-and-contexts` rejected.


== Out of scope

- Writes across a set: a three-character hash can match tasks in two projects.
- `-w` and `MINDTAPE_WORKTREE`, which stay one project.


== Dogfood

The owner reads the 30 projects under `~/projects/home`, this repo among them, with `mt @home`.

