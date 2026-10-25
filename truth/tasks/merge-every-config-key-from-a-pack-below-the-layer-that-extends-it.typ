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
  estimate: (src-files: 4),
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
  status: proposed(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on(
      "let-a-pack-carry-any-config-key-templates-and-typst-files.typ",
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

- 🟢 The pack's `[tasks.priority]` applies. A `[tasks.priority]` in the extending layer replaces it whole.
- 🟢 Two packs in one `extends` that set one key fail: the error names the key and both packs.
- 🟢 A pack that declares `extends` or `disable` fails, naming the pack and the key.
- 🟢 `mt config show --origin` prints `key <path>: pack <name>` for each key a pack set.
- 🟢 Rule arrays merge as before.


== Changes

- `crates/mindtape/src/config/packs_tests.rs`: the red tests first, one per TO-BE bullet.
- `crates/mindtape/src/config/pack_resolve.rs`: replace the allow-list in `validate_pack_shape` with a deny-list of `extends` and `disable`.
- `crates/mindtape/src/config/packs.rs`: fold each pack's other keys into a sub-layer just below its layer, with the two-pack conflict check.
- `crates/mindtape/src/config/layers.rs`: carry each key's origin for `--origin`.
- `crates/mindtape-cli/src/cmd/config.rs`: print the key origins after the rule origins.
- `www/content/rule-packs.typ`: rename to `packs.typ` at `/packs/`, and rewrite "What a pack may hold" and "Merge order".
- `www/content/*.typ`: point every `/rule-packs/` link at `/packs/`.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Let the later pack win on a shared key: the project would get one legend with no sign of which.
- Merge tables key by key across packs: two half-tables, such as priority bounds from one pack and bands from another, make no sense together.


== Out of scope

- `[[on.flip]]` merging by name: `merge-named-on-flip-entries-across-packs-and-layers.typ`.
- Directory packs: `let-a-pack-directory-ship-templates-and-typst-files.typ`.


== Dogfood

`ship-the-flow-and-ice-packs.typ` moves this repo's legend and priority into packs on top of this.

