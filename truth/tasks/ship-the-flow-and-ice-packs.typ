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
  status: proposed(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on(
      "name-packs-as-source-colon-pack.typ",
      "merge-named-on-flip-entries-across-packs-and-layers.typ",
      "resolve-param-in-a-packs-expressions-from-its-extends-entry.typ",
    ),
)

== Problem

+ Read the `[[tasks.status]]`, `[tasks.priority]` and `[[on.flip]]` entries of this repo's `.mindtape/config.toml`.
+ 🔴 Six statuses with their transitions, the ICE priority with three bands and the stale-WIP gate sit there by hand, and the bands repeat word for word in fourteen projects.


== TO-BE

```toml
extends = ["mindtape:tasks", "mindtape:flow", "mindtape:ice"]
```

- 🟢 `mindtape:flow` holds the six statuses from `proposed` to `failed` with their glyphs, colors, `terminal`, `unblocked` and `to` edges, and the `stale-wip` gate with `days = 30` as a parameter.
- 🟢 `mindtape:ice` holds `[tasks.priority]` as an ICE score on 0..9 with `allow-manual = false`, and the `low`, `normal` and `high` bands.
- 🟢 A project that declares its own `[[tasks.status]]` or `[tasks.priority]` replaces the pack's whole.
- 🟢 `www/content/packs.typ` lists both packs.


== Changes

- `crates/mindtape/src/config/packs_tests.rs`: the red tests first: each pack loads, the stale gate fires, a project legend replaces the pack's.
- `crates/mindtape/src/packs/flow.toml`, `crates/mindtape/src/packs/ice.toml` (new): the packs.
- `crates/mindtape/src/config/packs.rs`: register both.
- `.mindtape/config.toml`: extend both packs and drop the legend, the priority and the stale gate, keeping the close gate.
- `www/content/packs.typ`, `www/content/statuses.typ`, `www/content/priority.typ`: the two packs.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- The five-status legend of the `mt init` flow preset: fewer projects adopt it, and the preset can extend this pack later.
- The priority bands in `mindtape:tasks`: the six sections need no priority, so every adopter would get a scale it never asked for.


== Out of scope

- The `mmz` close gate: it runs a tool other projects may lack.
- The `mt init` presets.


== Dogfood

This repo's legend, priority and stale gate come from the two packs, and `mt flip --list` prints the same statuses as before.

