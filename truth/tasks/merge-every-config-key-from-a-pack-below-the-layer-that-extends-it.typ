#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Merge every config key from a pack below the layer that extends it",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("mt", "gating"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a pack's priority, fields and views apply with one `extends` line],
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
      "let-a-pack-carry-any-config-key-templates-and-typst-files.typ",
      "rename-the-rule-packs-page-to-packs.typ",
    ),
)

== Problem

+ Put `[tasks.priority]` in `./team.toml` and add `extends = ["./team.toml"]` to `.mindtape/config.toml`.
+ Run `mt check`.
+ 🔴 It prints `extends: pack '…/team.toml' may not declare: 'priority'`.


== TO-BE

```toml
# team.toml
[tasks.priority]
framework = "ice"
```

- 🟢 The pack's keys deep-merge into a sub-layer just below the layer that extends it, with `deep_merge` from `crates/mindtape/src/config/merge.rs`: the layer's `[tasks.priority] allow-manual = true` keeps the pack's `framework`, and the floor fills the rest. A `max = 5` there would fail the floor's bands.
- 🟢 Two packs in one `extends` that set one value differently fail, naming the dotted key and both packs, with a hint to set the key in the extending layer. Equal values, such as one `[dicts.source]` in two packs, agree.
- 🟢 The extending layer settles a conflict between its own packs: when it sets the dotted key, or a non-table above it, its value wins and nothing fails.
- 🟢 A pack declaring `extends`, `disable`, `project.code`, `tasks.roots`, `templates.dirs` or `[typst]` fails, naming the pack and the key. `project.code` is the project's identity, which a shared pack can't know.
- 🟢 Rule arrays merge as before.
- 🟢 A pack's `[go]` or `[add]` editor is listed by `allow::commands`, so it waits for `mt allow` like the project's own.


== Changes

- `crates/mindtape/src/config/packs_tests_merge.rs` (new, as `packs_tests.rs` sits near its cap) and `packs_tests_extends.rs`: the red tests first, one per TO-BE bullet. `crates/mindtape/src/allow_tests.rs` gets the editor one: the gate reads the merged config, so it needs no code, only the test, which can't be written until a pack may carry `[go]`.
- `crates/mindtape/src/config/pack_resolve.rs`: replace the allow-list in `validate_pack_shape` with the deny-list above.
- `crates/mindtape/src/config/pack_fold.rs` (new): one layer's packs folded into one sub-layer, with the conflict check.
- `crates/mindtape/src/config/packs.rs`: take each pack's rule arrays out, fold the rest against the layer's own table, and merge the sub-layer below the layer.
- `crates/mindtape/src/config/packs_toml.rs`: `get_path`, for the deny-list.
- `crates/mindtape/src/config/mod.rs`: the `extends` field doc on what a pack may hold, and a `hint` on `ConfigError::Invalid`.
- `crates/mindtape-cli/src/diag/from_lib.rs`: print that hint as `hint:`.
- `schema/config.schema.json`: regenerate, since the schema copies that doc.
- `www/generate-packs.sh`: capture the refusal of `extends` in place of `priority`, which no longer fails, a pack's priority merged below the project's, and the two-pack conflict.
- `www/content/packs.typ`: rewrite "What a pack may hold" and "Merge order".
- `www/content/config.typ` and the pages linking `/packs/` as "Rule packs": name the page Packs.
- `CHANGELOG.md`: an entry under `[Unreleased]`, and the packs entry no longer says a pack holds only rules.


== Alternatives

- The layer's table replaces the pack's whole: this repo's `[tasks.estimate.queries]` would vanish beside a pack's `[tasks.estimate]`.
- Let the later pack win on a shared value: nothing would say which one the project got.


== Out of scope

- `[[on.flip]]` merging by name: `merge-named-on-flip-entries-across-packs-and-layers.typ`.
- Directory packs: `read-a-directory-pack-and-ship-its-templates.typ`.
- Key origins in `mt config show --origin`: `print-each-pack-key-and-parameter-origin-in-mt-config-show.typ`.


== Dogfood

`ship-the-flow-and-ice-packs.typ` moves this repo's legend and priority into packs on top of this.

