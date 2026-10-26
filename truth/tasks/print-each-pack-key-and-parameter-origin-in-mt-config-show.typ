#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print each pack key and parameter origin in mt config show",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "cli"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a project sees which pack set a value before it overrides it],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on(
      "merge-every-config-key-from-a-pack-below-the-layer-that-extends-it.typ",
      "resolve-param-in-a-packs-expressions-from-its-extends-entry.typ",
    ),
)

== Problem

+ Extend a pack that sets `[tasks.priority] framework = "ice"` and declares `[params] max-files = 5`, with `with = { max-files = 8 }`.
+ Run `mt config show --origin`.
+ 🔴 It prints one line per rule and nothing for the priority key or the parameter, so the reader can't tell which pack set the scale or which cap applies.


== TO-BE

```console
$ mt config show --origin
rule tasks.rules.judgment-stays-small: pack /work/app/team.toml
key tasks.priority.framework: pack /work/app/team.toml
param max-files = 8: project, for pack /work/app/team.toml
```

- 🟢 Each leaf value a pack set prints as `key <dotted path>: pack <origin>`, after the rule lines, naming the pack as the rule lines already do. A value the layer replaced prints the layer.
- 🟢 Each parameter prints its value and where it came from: `project` or another layer for a `with` value, `default` otherwise.


== Changes

- `crates/mindtape-cli/src/cmd/config_tests.rs`: the red test first, the output above.
- `crates/mindtape/src/config/packs.rs`, `crates/mindtape/src/config/pack_fold.rs`: record the pack behind each leaf value and each parameter while folding. `PackLayer::into_table` drops the per-pack tables, so return them beside the merged sub-layer.
- `crates/mindtape/src/config/params.rs`: `apply_params` resolves each parameter in `resolve_values` and drops the result, so return each value and whether the entry's `with` set it.
- `crates/mindtape/src/config/layers.rs`: carry those origins beside the rule origins.
- `crates/mindtape-cli/src/cmd/config.rs`: print them.
- `crates/mindtape/src/config/mod.rs`, `crates/mindtape/src/lib.rs`: follow the wider `load_config_layered` result and re-export the new origin type.
- `www/content/packs.typ`: the `--origin` section.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- A separate `mt config origin` command: `--origin` already answers where a value came from, for rules today.


== Out of scope

- Origins of a layer's own keys, without packs: every value there comes from the layer `mt config show` already names.


== Dogfood

This repo's `mt config show --origin` names the pack behind its legend, priority and caps.

