#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Replace the task section rules with property gates",
  kind: "decision",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 6,
  tags: ("mt", "docs"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the rules follow the change, not the commit type],
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
  status: wip(2026, 11, 1),
  links: depends-on("add-mt-tags.typ"),
)


== Problem

+ Run `mt rules --for add` and count the sections a `dev` task must carry.
+ 🔴 A `dev` task carries `Rollback` for a revert that is the default, and a `decision` task cannot carry `Alternatives` until its kind is `feat`.
+ 🔴 The 281 closed `dev` tasks carry a `Rollback` section, and 251 of them say revert the commit.

== TO-BE

- 🟢 The section rules follow properties: `stateful` requires Rollback, `stateless` forbids it, and `decision` carries Alternatives.
- 🟢 `decision` is a kind, so a decision closes in a task-only commit without a tag: `flip-with-work` reads `staged(kind == "decision")`.
- 🟢 `Problem` and `AS-IS` both pass the allowlist during the transition, and `AS-IS` gets the same unique, order, non-empty and word-cap rules as `Problem`.
- 🟢 The statefulness and example rules land behind the pack params `statefulness` and `examples`, both off, so the old templates and fixtures stay green until `turn-on-state-rules` sets them.

== Changes

- `crates/mindtape/src/packs/tasks.toml`: the `decision` kind, `alternatives-for-design`, the AS-IS rule set, the statefulness and example rules behind `param("statefulness")` and `param("examples")`, the both-heading allowlist and the `flip-with-work` kind test.
- `crates/mindtape/src/packs/dogfood.toml`: the matching allowlist.
- `.mindtape/config.toml`: the tag list, the `decision` tag removal, and the two params set false.
- `truth/tasks/*.typ`: the filed tasks' `stateless` tags and this task's own Rollback removal.

== Alternatives

- Gate by commit type, as today: a `dev` chore carries a Rollback for a default revert, and a decision cannot carry Alternatives until it is a feat.
- Turn every rule on at once: the templates, the api-service fixture and the open jscpd task fail the gate before the corpus is migrated.

== Out of scope

- The templates and fixtures in `task-templates` and `task-shape-docs`, the AS-IS sweep in `as-is-sweep`, the allowlist narrowing in `drop-problem-heading`, the switch-on in `turn-on-state-rules` and the x-goals work in `prelude-verdicts`.

== Dogfood

This repo's own backlog adopts the pack and the rules it fixes.
