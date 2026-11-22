#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Ship the goal DSL in the prelude",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  links: surfaced-by("rewire-goals-through-typst-markers.typ")
    + depends-on(
      "define-the-goal-dsl-over-metadata-markers.typ",
      "migrate-the-open-backlog-and-switch-the-goal-rule.typ",
    ),
  tags: ("found", "tooling", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 22),
)

== AS-IS

+ Reviewing the goals milestone's cost to an adopter
+ 🔴 #raw("truth/goal-dsl.typ"): the goal DSL is project Typst. An adopter copies five files instead of three

```
truth/goal-dsl.typ
truth/goal-calls.typ
truth/goals.typ
truth/process-goals.typ
truth/lib.typ
```

== TO-BE

- 🟢 `lib/prelude.typ` defines `Helps`, `Hinders`, `Neutral`, `goal`, `adopter`, `maintainer` and `check-goal-over`. An adopter imports them.
- 🟢 The project keeps only its registries, `goal-calls.typ` and `lib.typ`.

```typ
#import "@local/mindtape:0.5.0": Helps, Hinders, Neutral, goal, check-goal-over
```

== Changes

- `lib/prelude.typ`, `lib/typst.toml`: the DSL and a 0.5.0 bump. `check-goal-over(product, process, markers)` takes the two registries, because a project's goals are its own.
- `crates/mindtape/src/loader/preludes/0.5.0/prelude.typ`: the snapshot.
- `crates/mindtape/src/loader/embedded.rs`: `PRELUDE_VERSION` to 0.5.0, `PRELUDE_SUPPORTED` gains it, `PRELUDE` reads the 0.5.0 snapshot, and `PRELUDE_0_4` joins `PRELUDE_SNAPSHOTS`. Without this, `mt fix` still calls 0.4.0 current and refuses a 0.5.0 import as newer (`crates/mindtape/src/prelude_import.rs:146`, `:152`).
- `crates/mindtape/src/loader/embedded_tests.rs`: the supported set (`:98`), the in-tree copy (`:142`) and the manifest mirror (`:243`).
- `truth/goal-dsl.typ`: deleted, its functions in the prelude.
- `truth/goal-calls.typ`, `truth/goals.typ`, `truth/process-goals.typ`, `truth/lib.typ`: import from the prelude. `truth/lib.typ` wraps `check-goal-over` with its two registries.
- `www/content/dsl.typ`, `www/content/tie-work-to-goals.typ`: the prelude functions.
- `CHANGELOG.md`: an entry under `[Unreleased]`.

== Out of scope

- The registries. A project's goals are its own.
- A pack. A task cannot import a pack's Typst file, because the task's world roots at the project.

== Dogfood

This repo's `truth/goal-dsl.typ` disappears, and its registries import `goal` from the prelude.

== Goals

+ #Helps(adopter, to: guide-the-implementer)[the DSL moves to the prelude. An adopter imports it instead of copying it].
+ #Helps(adopter, to: keep-the-engine-agnostic)[the generic DSL leaves the project, and only the registries stay project-side].
+ #Helps(adopter, to: adopt-in-one-line)[an adopter copies three files instead of five, and gets later DSL fixes from the prelude].
+ #Helps(maintainer, to: match-docs-to-code)[the prelude page and the goals page name the same functions].
+ #Neutral(read-tasks-at-a-glance, run-checks-at-the-write, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe).
