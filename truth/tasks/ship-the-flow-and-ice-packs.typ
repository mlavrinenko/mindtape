#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Ship the flow and ice packs",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("mt", "gating"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[the status legend, its stale-WIP gate and the ICE priority adopt in two lines],
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
      "ship-the-dogfood-pack-and-widen-mindtape-tasks.typ",
      "merge-named-on-flip-entries-across-packs-and-layers.typ",
    ),
)

== Problem

+ Read the `[[tasks.status]]`, `[tasks.priority]` and `[[on.flip]]` entries of this repo's `.mindtape/config.toml`.
+ 🔴 The six statuses of the embedded floor sit there again by hand only to add `to` edges and `unblocked`, beside an ICE `framework` that fourteen projects copy and a stale-WIP gate that eight copy.


== TO-BE

```toml
extends = ["mindtape:tasks", "mindtape:flow", "mindtape:ice"]
```

- 🟢 `mindtape:flow` restates the floor's six statuses with this repo's `to` edges and `unblocked` flags, since an array replaces whole, and adds the `stale-wip` gate with `days = 30` as a parameter. A project legend with no `wip` leaves that gate naming a status it lacks, so it lists `stale-wip` in `disable`.
- 🟢 `mindtape:ice` sets only `framework = "ice"`, `allow-manual = false`, `required = "!terminal"` and `example = 3` under `[tasks.priority]`. The floor's 0..9 scale and its three bands show through.
- 🟢 A project's own `[[tasks.status]]` replaces the pack's legend whole, and `mt check` validates the `to` edges of whichever legend wins.


== Changes

- `crates/mindtape/src/config/packs_tests_flow.rs`, `crates/mindtape-cli/tests/it/flow_pack_e2e.rs` (new, since `packs_tests.rs` nears its cap): the red tests first: each pack loads, the stale gate fires, a project legend replaces the pack's.
- `crates/mindtape/src/packs/flow.toml`, `crates/mindtape/src/packs/ice.toml` (new): the packs.
- `crates/mindtape/src/config/pack_registry.rs`, `Justfile`: register both, and add both files to `TOML_FMT_PATHS`.
- `.mindtape/config.toml`: extend both packs and drop the legend, the priority and the stale gate, keeping the close gate.
- `www/content/packs.typ`, `statuses.typ`, `priority.typ`, `hooks.typ`, `www/generate-examples.sh`: the two packs, with the stale gate sliced from the pack file.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- The transitions in the floor itself: a project with no transition graph would get one it never asked for.
- The priority framework in `mindtape:tasks`: the six sections need no priority.


== Out of scope

- The `mmz` close gate: it runs a tool other projects may lack.
- The `mt init` presets.


== Dogfood

This repo's legend, priority and stale gate come from the two packs, and `mt flip --list` prints the same statuses as before.

