#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show every tracked project from one mt @name",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 3,
  ),
  difficulty: 3,
  kind: "milestone",
  tags: ("cli",),
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
  status: done(2026, 10, 17),
  links: related(
    "read-many-projects-through-one-mt-name-selector.typ",
  )[the design its children build],
)

== Problem

+ Keep Mindtape tasks in 30 repos under `~/projects/home`.
+ Ask which of them has work ready, blocked or broken.
+ 🔴 `mt` answers for one repo per run: `mt @home ls` prints `error: unknown worktree: '@home'`, and a shell loop takes 2.9s and prints tables with no project name.


== TO-BE

- 🟢 `mt @home` prints one row per project, `mt @home ls ready` lists their tasks grouped by project, and `mt @home check` checks them all.
- 🟢 `~/.config/mindtape/at.toml` is the one place that names projects and sets.
- 🟢 Every project loads in parallel, under its own config.


== Changes

- `truth/tasks/*.typ`: the child tasks, linked with `parent` to this one. This milestone changes no file itself.


== Out of scope

- Writes across a set, and a TUI or web dashboard.


== Dogfood

The owner reads this repo beside the 29 other projects under `~/projects/home` with `mt @home`.

