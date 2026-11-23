#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Derive the sweep goal list from the registry",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  links: (
    (
      task: "rewire-goals-through-typst-markers.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "tooling", "gating", "stateless", "no-ui-ux"),
  assignee: "agent",
  status: proposed(2026, 11, 22),
)

== AS-IS

+ Reviewing the goal DSL milestone after it landed.
+ 🔴 #raw("contrib/ask-refactoring.sh"): the sweep hard-codes the ten goal names, so a new or renamed goal makes every sweep's `mt add` fail.

`goals_with_one_helps` lists the product and process names in shell, and the five arms pass its output as `--set goals=`. No gate compares that list with the registries.

== TO-BE

- 🟢 The sweep reads the goal names from the project registry or a checked manifest.
- 🟢 A gate fails when the list and the registries disagree, before a sweep files anything.
- 🟢 A new goal needs one edit, not a shell hunt.

```sh
goals_with_one_helps maintainer keep-changes-safe 'why'
```


== Changes

- `contrib/ask-refactoring.sh`: derive the neutral list from a project source or a manifest.
- `crates/mindtape-cli/tests/it/` or a `just check` gate: compare the list with `truth/goals.typ` and `truth/process-goals.typ`.

== Out of scope

- The goal DSL move. The sweep still needs the project's names.

== Dogfood

The next goal addition cannot break the advisory sweep's filing path.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[the sweep cannot drift from the goal registry].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).

