#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop the Problem heading from the task allowlist",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 2,
  tags: ("mt", "stateless"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[one heading is accepted, not two],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 1),
  links: depends-on("as-is-sweep.typ"),
)


== AS-IS

+ Run `mt rules --for add` and read the allowlist.
+ 🔴 The allowlist still accepts `Problem` after the sweep, so a new task can pick either heading.
+ 🔴 A reopened old task keeps `Problem` and passes without a rename.

== TO-BE

- 🟢 `Problem` is gone from both allowlists and only `AS-IS` is accepted.
- 🟢 A task with `Problem` is refused with a message that names `AS-IS`.
- 🟢 `mt check` stays green on the swept backlog.

```toml
allowed = [{ match = "^(AS-IS|Environment|TO-BE|Source of truth|Changes|Alternatives|Rollback|Out of scope)$" }]
```

== Changes

- `crates/mindtape/src/packs/tasks.toml`: the allowlist and the remove-Problem message.
- `crates/mindtape/src/packs/dogfood.toml`: the matching allowlist.

== Out of scope

- The corpus in `as-is-sweep`, the docs in `task-shape-docs` and the templates in `task-templates`.

== Dogfood

This repo's next reopened task is refused until it says `AS-IS`.
