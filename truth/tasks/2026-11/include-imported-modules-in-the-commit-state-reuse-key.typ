#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Include imported modules in the commit state reuse key",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 3,
  ),
  difficulty: 5,
  links: (
    (
      task: "rewire-goals-through-typst-markers.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "tooling", "stateless", "no-ui-ux"),
  assignee: "agent",
  status: done(2026, 11, 23),
)

== AS-IS

+ Reviewing the goal DSL milestone after it landed.
+ 🔴 #raw("crates/mindtape/src/commit/state_cache.rs"): the commit state cache reuses a task by source blob and mode, but its markers depend on imported project modules.

`lookup` checks `seen.source == source` and `fits` tags/depth. `load_env` includes config, roots and packages, not `truth/goals.typ`. A source-identical task across a goal-registry change can replay stale markers.

== TO-BE

- 🟢 A reused state carries the markers its own tree's imports produce, or the state re-evaluates the task when an imported module changed.
- 🟢 The reuse key includes the project modules the task's harvest read, the way the content cache's deps already do.
- 🟢 A test changes an imported module without touching the task and asserts the state's markers move.

```text
state A: truth/goals.typ hash 1, task.typ hash X
state B: truth/goals.typ hash 2, task.typ hash X
```


== Changes

- `crates/mindtape/src/commit/state_cache.rs`, `state_loader.rs`: fold the imported module hashes into `load_env`/`lookup` or recompute markers.
- `crates/mindtape/src/commit/state_loader_tests.rs`: the registry-change case.

== Out of scope

- A live goal rule on commit states. The commit rules ignore markers today, and this closes the latent path.

== Dogfood

`mt check --commits` and the commit hook cannot replay a marker from another tree.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a replayed state cannot carry stale markers].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).

