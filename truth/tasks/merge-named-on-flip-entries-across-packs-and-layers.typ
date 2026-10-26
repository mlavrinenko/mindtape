#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Merge named on.flip entries across packs and layers",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("mt", "gating"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a pack's flip gates apply beside the project's own, and the project replaces or drops one by name],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on(
      "merge-every-config-key-from-a-pack-below-the-layer-that-extends-it.typ",
    ),
)

== Problem

+ Put an `[[on.flip]]` stale-WIP gate in a `.mindtape/defaults.toml` one directory above a project whose `.mindtape/config.toml` has an `[[on.flip]]` close gate.
+ Run `mt config show` in the project.
+ 🔴 Only the close gate is left: `on.flip` replaces whole per layer, and `FlipHook` in `crates/mindtape/src/config/hooks.rs` has no `name` to replace or `disable` one entry by. A pack's gates would vanish the same way.


== TO-BE

```toml
[[on.flip]]
name = "stale-wip"
when = 'into("wip") && stale(30)'
actions = [{ gate = true, message = "re-read the scope first" }]
```

- 🟢 `[[on.flip]]` takes an optional `name`, and its entries append across packs and layers as `[[tasks.rules]]` entries do.
- 🟢 A later entry with the same `name` replaces the earlier one in place, and `disable = ["stale-wip"]` drops it.
- 🟢 An unnamed entry appends and can't be replaced. Two entries of one file sharing a name are a config error.
- 🟢 `mt config show --origin` labels a hook `on.flip.stale-wip`, or `on.flip[<index>]`. `mt rules` lists each action under its hook's label and slot: `on.flip.stale-wip.actions[0]`.


== Changes

- `crates/mindtape/src/config/packs_tests_flip.rs` (new) and `active_rules_tests.rs`: the red tests first: an ancestor's gate fires beside the project's, a same-name entry replaces it, `disable` drops it, an unnamed one appends, a pack's gate merges, a duplicate name in one file fails, the labels.
- `crates/mindtape/src/config/hooks.rs`: `name: Option<String>` on `FlipHook`, and `validate_flip_names`, called from `semantic.rs`.
- `crates/mindtape/src/config/packs.rs`: `on.flip` joins the arrays that merge by name, so `disable` reaches it too.
- `crates/mindtape/src/config/active_rules.rs`: label an action by its hook's name, and keep its slot, so two actions of one hook stay apart.
- `crates/mindtape-cli/tests/it/rules_list_e2e.rs`: `--origin` now lists the hooks, so the label test compares them too.
- `.mindtape/config.toml`: name this repo's two hooks.
- `schema/config.schema.json`: regenerate.
- `www/content/hooks.typ`, `www/content/packs.typ`, `www/content/config.typ`: the `name` key and the merge.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Require `name` on every entry: `[[tasks.rules]]` lets an unnamed one append, and one rule for both is simpler.
- Keep `on.flip` replacing whole: a pack's gate would vanish the moment a project adds one.


== Out of scope

- `[[tasks.waits]]` and the other arrays: none is shared across projects yet.


== Dogfood

This repo names its two hooks, `stale-wip` and `gates-fresh`. The stale-WIP gate moves into `mindtape:flow` once that pack exists, and the close gate stays local, both firing on one flip.

