#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Correct the stale goal migration claims",
  kind: "dev",
  priority: ice(
    impact: 2,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 1,
  links: (
    (
      task: "rewire-goals-through-typst-markers.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "docs", "tooling", "stateless", "no-ui-ux"),
  assignee: "agent",
  status: done(2026, 11, 23),
)

== AS-IS

+ Reviewing the goal DSL milestone after it landed.
+ 🔴 #raw("truth/tasks/2026-11/migrate-the-open-backlog-and-switch-the-goal-rule.typ"): the migration record says `truth/lib.typ` narrows to `0.3.0` while it imports `0.5.0`, and the changelog places the old-name aliases in the registries.

`truth/lib.typ` imports `@local/mindtape:0.5.0`, and the aliases live in `truth/goal-calls.typ`. `CHANGELOG.md` says each registry keeps the old `x-goals` key as an alias.

== TO-BE

- 🟢 The migration record states the version the code imports.
- 🟢 The changelog says where the aliases live, or drops them with the backward-compatibility removal.

```text
truth/lib.typ imports @local/mindtape:0.5.0
CHANGELOG.md: the aliases live in truth/goal-calls.typ
```


== Changes

- `truth/tasks/2026-11/migrate-the-open-backlog-and-switch-the-goal-rule.typ`: correct the version.
- `CHANGELOG.md`: correct the alias sentence, or remove it with the old-API removal.

== Out of scope

- The code. This is a record correction.

== Dogfood

A reader of the migration history sees the same version and alias location the tree has.

== Goals

+ #Helps(maintainer, to: match-docs-to-code)[the migration record matches the code].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe).

