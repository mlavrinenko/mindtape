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
  status: proposed(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ"),
)

== Problem

+ Put `[tasks.priority]` in `./team.toml` and add `extends = ["./team.toml"]` to `.mindtape/config.toml`.
+ 🔴 `mt check` prints `extends: pack '…/team.toml' may not declare: 'priority'`: `validate_pack_shape` in `crates/mindtape/src/config/pack_resolve.rs` allows only rule arrays and `[dicts]`.
+ 🔴 A pack is one TOML file, so the `.mindtape/templates/*.typ` files and the Typst file a `check = { typst = … }` rule reads get copied from project to project instead.


== TO-BE

```text
team.toml            # a file pack
team/pack.toml       # a directory pack
team/templates/feat.typ
team/lib.typ         # check = { typst = "lib.typ", … } resolves here
```

- 🟢 A pack declares any config key but `extends`, `disable` and `[pack-sources]`, so packs stay one level deep.
- 🟢 A pack's key sits just below the layer that extends it, and the layer's own key replaces it whole. Two packs in one `extends` setting one key is an error naming both.
- 🟢 Rule arrays and `[[on.flip]]` append and replace by `name`. `[[tasks.status]]` replaces whole.
- 🟢 A directory pack's `templates/` joins the template dirs after the project's own, and its paths resolve from the pack directory, wherever it sits.


== Changes

- `truth/tasks/*.typ`: the leaves that build it, linked with `parent` to `share-config-between-projects-through-packs.typ`. This decision changes no file itself.


== Alternatives

- A second mechanism, such as a preset layer, for the keys that aren't rules: two ways to share one config.
- Last pack wins on a shared key: nothing would say which legend the project got.
- Merge `[[tasks.status]]` entry by entry: halves of two transition graphs make no graph.
- Require `name` on every `[[on.flip]]` entry: `[[tasks.rules]]` already lets an unnamed entry append, and one rule for both is simpler.


== Out of scope

- A pack extending a pack: one level keeps resolution and `mt config show --origin` simple.
- A Typst file the task files import, such as `truth/lib.typ`: a task imports it by a path inside the project, so sharing it means publishing a Typst package.


== Dogfood

This repo's status legend, priority and `on.flip` gates move into the official packs that `ship-the-shared-practice-as-official-mindtape-packs.typ` settles.

