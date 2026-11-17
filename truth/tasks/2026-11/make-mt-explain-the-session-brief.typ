#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Make mt explain the session brief",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  kind: "milestone",
  tags: ("cli", "docs", "stateless"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: helps[the index reads as the project's shape, not a bare list.],
      implementable-tasks: helps[a session starts without reading `mt config show` or a stale example.],
      enforced-practice: helps[the forbidden commands show before a flip, not after.],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the CLI answers what the project is, not only what it checks.],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[AGENTS.md stops repeating what `mt explain` prints.],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 17),
)

== AS-IS

+ Run `mt explain` in this repo.
+ 🔴 It lists the rules, tags, views and dicts. It omits the project's roots, statuses, actors and `rm` policy. It also omits the task's priority, difficulty, estimate, kinds, links, comments, extensions and templates, and the commit trailer.
+ Run `mt add --list`.
+ 🔴 It lists 16 bare names. `mt explain` never mentions templates. An agent tries `mt explain templates` and gets an unknown argument.
+ Run `mt ls`.
+ 🔴 Its footer says `default view` but not the filter.
+ Read `AGENTS.md`.
+ 🔴 Its task example repeats the shape `mt explain` should own, and its `estimate: (days: 1, hours: 2)` is illegal here, where this repo allows only `src-files`.

== TO-BE

- 🟢 `mt explain` prints `project:`, `task:` and `commits:` blocks: roots, statuses, actors, `rm`, priority, difficulty, estimate, kinds, links, comments, extensions, templates and the trailer, plus an `absent:` line for unconfigured keys.
- 🟢 It prints a `refusals:` block for the commands the user has not allowed. `mt explain templates` lists each template with kind and slots. `mt add --list` retires in favor.
- 🟢 `mt ls` names the filter it applied in its footer, or the view's description when it has one.
- 🟢 `AGENTS.md` points at `mt explain` for the shape, and the skill's recipe is `mt explain`, `mt ls`, `mt find`, `mt add --describe`, `mt add`, `mt check`, then `mt explain rules --for <moment>`.

== Changes

- `truth/tasks/2026-11/show-the-projects-shape-in-mt-explain.typ`: the `project:`, `task:`, `commits:` and `absent:` blocks.
- `truth/tasks/2026-11/show-only-the-forbidden-gates-in-mt-explain.typ`: the `refusals:` block.
- `truth/tasks/2026-11/list-the-templates-in-mt-explain.typ`: the `templates` listing.
- `truth/tasks/2026-11/retire-mt-add-list.typ`: the `mt add --list` retirement.
- `truth/tasks/2026-11/show-the-applied-filter-in-the-mt-ls-footer.typ`: the `mt ls` footer.
- `truth/tasks/2026-11/cut-agents-md-to-what-mt-explain-cannot-say.typ`: the docs dedup.

== Out of scope

- Counts and the next task. `mt ls` and `mt board` own the state.
- The slots of each template. `mt add -t <name> --describe` owns them.
- The pack origins. `mt config show --origin` owns them.
- A `--now` flag. The recipe chains `mt ls` instead.
- The `[[on.rm]]` hook. It is a separate task, not part of the brief.

== Dogfood

This repo runs `mt explain` first in a session, then `mt ls`, and `mt explain rules --for flip` before a flip. `AGENTS.md` stops repeating the shape.
