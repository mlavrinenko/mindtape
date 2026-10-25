#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Let a pack carry any config key, templates and Typst files",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("mt", "gating", "decision"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a pack carries the legend, priority, fields, views and gates, not just the rules],
      typst-over-markdown: neutral,
      plain-files: neutral,
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
    26,
  )[agreed: a pack deep-merges as a layer just below the one extending it, unequal values from two packs conflict, and a directory pack ships templates and check files],
  links: parent("share-config-between-projects-through-packs.typ"),
)

== Problem

+ Put `[tasks.priority]` in `./team.toml` and add `extends = ["./team.toml"]` to `.mindtape/config.toml`.
+ 🔴 `mt check` prints `extends: pack '…/team.toml' may not declare: 'priority'`: `validate_pack_shape` in `crates/mindtape/src/config/pack_resolve.rs` allows only rule arrays and `[dicts]`.
+ 🔴 A pack is one TOML file, so the `.mindtape/templates/*.typ` files and the Typst file a `check = { typst = … }` rule reads get copied from project to project instead.


== TO-BE

```text
team.toml               # a file pack
team/pack.toml          # a directory pack
team/templates/feat.typ
team/check.typ          # check = { typst = "check.typ", … } resolves here
```

- 🟢 A pack merges like a config layer just below the layer that extends it: tables merge key by key, arrays and scalars replace whole, and the embedded floor shows through.
- 🟢 Two packs in one `extends` that set one value differently fail, naming the key and both packs. Equal values agree.
- 🟢 Rule arrays and `[[on.flip]]` append and replace by `name`.
- 🟢 A pack refuses `extends`, `disable`, `[pack-sources]`, `tasks.roots`, `templates.dirs` and `[typst]`.
- 🟢 A directory pack's `templates/` joins the template dirs after the merge. Its check and dict files resolve from the pack directory, wherever it sits.


== Changes

- `truth/tasks/*.typ`: the leaves that build it, linked with `parent` to `share-config-between-projects-through-packs.typ`. This decision changes no file itself.


== Alternatives

- A preset layer for the keys that aren't rules: two ways to share one config.
- The layer's table replaces the pack's whole: this repo's `[tasks.estimate.queries]` would vanish beside a pack's `[tasks.estimate]`.
- Last pack wins on a shared value: nothing would say which one the project got.
- Merge `[[tasks.status]]` entry by entry: halves of two transition graphs make no graph.


== Out of scope

- A pack extending a pack: one level keeps resolution and `mt config show --origin` simple.
- The keys refused above: they name the project's own tree, or a second level of packs.
- A Typst file the task files import, such as `truth/lib.typ`: a task imports it by a path inside the project, so sharing it means publishing a Typst package.


== Dogfood

This repo's status legend, priority and `on.flip` gates move into the official packs that `ship-the-shared-practice-as-official-mindtape-packs.typ` settles.

