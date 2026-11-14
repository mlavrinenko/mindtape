#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Show on.add hooks and their origin in mt explain",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("mt", "cli", "stateless", "no-repro"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[mt explain add lists every hook, its origin and its actions],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 11),
  links: depends-on(
    "add-the-on-add-hook-shape-validation-and-merge.typ",
    "let-rules-and-templates-choose-where-new-tasks-land.typ",
  ),
)

== AS-IS

+ Add an `[[on.add]]` hook in a pack and in the project.
+ 🔴 `mt explain` lists flip hooks and rules, but no add hook. A placement then comes from nowhere.
+ `mt config show --origin` has no entry that states the hook's source.


== TO-BE

- 🟢 `mt explain add` lists each hook in declaration/merged order, with its `when`, actions and origin.
- 🟢 `mt config show --origin` labels the `on.add` entry that supplied a hook.
- 🟢 A hook a nearer layer replaced or disabled is absent from both lists. The report then matches what runs.

```
$ mt explain add
default  -                      dir = "tasks/{kind}"
fixes    template == "fix"      dir = "tasks/fixes"
```


== Changes

- `crates/mindtape/src/config/active_add.rs`: flatten hooks with their origin for explain.
- `crates/mindtape-cli/src/cmd/explain/add.rs`: render the add-hook list.
- `crates/mindtape-cli/src/cmd/explain/mod.rs`: register `mt explain add`.
- `crates/mindtape-cli/tests/it/explain_e2e.rs`: origin, replaced and disabled hooks, and the rendered list.
- `www/content/hooks.typ`, `www/content/cli/explain.typ`: document the new listing.


== Alternatives

- `mt explain rules --for add`: forces effects into the rule model, with severity and gates they do not have.
- A flat list without origin leaves a hook with no source as a placement from nowhere.
- `mt explain hooks` for both events: broader, and `mt explain add` names the event whose actions differ.


== Out of scope

- The merge, named replacement and `disable` stay out. The config task owns them.
- The derive function contract stays out. It is a separate task.
- A per-hook explain detail page stays out. The list is enough.
- The rules list and its `--for` moments.


== Dogfood

This repo's packs can provide a default placement hook, and `mt explain add` shows where it came from.

