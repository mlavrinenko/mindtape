#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Ship the wip-caps and budget packs",
  kind: "feat",
  priority: ice(
    impact: 7,
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
      enforced-practice: helps[one open leaf and a capped judgment task hold the same way in every adopting project],
      cheap-adoption: helps[the WIP caps and the size budget adopt in one line each, with the caps as parameters],
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
      "name-packs-as-source-colon-pack.typ",
      "resolve-param-in-a-packs-expressions-from-its-extends-entry.typ",
    ),
)

== Problem

+ Read the `[[tasks.rules]]` of this repo's `.mindtape/config.toml`.
+ 🔴 `milestone-wip-cap`, `leaf-wip-cap`, `judgment-stays-small` and `src-budget` sit there by hand, and nine other projects copy them, three with a different cap or `when`.


== TO-BE

```toml
extends = [
  "mindtape:tasks",
  "mindtape:wip-caps",
  { pack = "mindtape:budget", with = { max-files = 8 } },
]
```

- 🟢 `mindtape:wip-caps` holds the two WIP caps, with `milestones = 1` and `leaves = 1` as parameters.
- 🟢 `mindtape:budget` holds `judgment-stays-small`, skipping milestones, and `src-budget`, with `min-difficulty = 3` and `max-files = 5` as parameters. It sets `[tasks.estimate] units = ["src-files"]`, a query `src-files = "@source"` and a `[dicts.source]` of `**`.
- 🟢 A project's `[tasks.estimate.queries]` and its own `[dicts.source]` merge over the pack's, as this repo's do.


== Changes

- `crates/mindtape/src/config/packs_tests_budget.rs` (new, since `packs_tests.rs` nears its cap): the red tests first: each pack's rules load with their defaults and with `with` values.
- `crates/mindtape/src/packs/wip-caps.toml`, `crates/mindtape/src/packs/budget.toml` (new): the packs.
- `crates/mindtape/src/config/pack_registry.rs`, `Justfile`: register both, and add both files to `TOML_FMT_PATHS`.
- `.mindtape/config.toml`: extend both packs and drop the four hand-written rules and `[tasks.estimate] units`.
- `www/content/packs.typ`: both packs' rules and parameters, read from the generated pack files.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- One pack for caps and budget: a project that wants the caps but sizes tasks in `files` would have to `disable` half of it.


== Out of scope

- An estimate unit as a parameter: a parameter reaches expressions only, so a project sizing in `files` overrides the two budget rules by name.


== Dogfood

This repo's caps and budget come from the two packs, and `mt config show --origin` reads `pack mindtape:wip-caps` and `pack mindtape:budget` for them.

