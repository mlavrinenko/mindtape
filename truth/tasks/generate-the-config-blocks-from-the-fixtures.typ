#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Generate the config blocks from the fixtures",
  kind: "docs",
  status: done(2026, 9, 22),
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 3,
  ),
  difficulty: 2,
  tags: ("www", "docs", "tooling"),
  estimate: (files: 12),
  assignee: "agent",
  x-goals: (
    product: (
      readable-tasks: helps[an example the checker runs cannot drift from the checker],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[config blocks shown on the site come out of checked fixtures],
    ),
  ),
)

== Problem

1. 🔴 The audit's hand-typed blocks are config, not task markers, so
  `www/generate-examples.sh` cannot slice them: `/named-sets/`'s views and
  dictionaries, `/hooks/`'s gate pair, `/estimates/`'s `files` bound,
  `/schema/`'s full record and `/rule-packs/`'s adopt block are copies of
  fixture or repo config with no check behind them.

== TO-BE

- 🟢 `generate-examples.sh` slices declared config halves (`toml` keys or whole
  sections) from the fixtures, with the same marker discipline task slices use.
- 🟢 Each page renders its block from the generated fact; the `files`-unit
  bound lands in a checked example (learn-piano or a new `examples/estimates`).
- 🟢 `/schema/`'s record comes from `www/generated/ls-task.json` or an
  extras-bearing slice of a fixture.

== Source of truth

- `examples/learn-piano/.mindtape/config.toml`, the repo's own
  `.mindtape/config.toml`, and `www/generated/ls-task.json`.
- `www/generate-examples.sh` — the slicing pattern.

== Changes

- `www/generate-examples.sh` — slice config halves.
- `www/content/named-sets.typ`, `hooks.typ`, `estimates.typ`, `schema.typ`,
  `rule-packs.typ` — render the slices.
- `examples/learn-piano/.mindtape/config.toml` and the repo's own
  `.mindtape/config.toml` — where a bound or hook must be added.
- `outdatty.yaml` and `Justfile` — new fixture/arm wiring.

== Out of scope

- Deriving the built-in priority tables; filed separately.

== Dogfood

`mt check` on every fixture the slices come from, `check-doc-facts`, and the
normal gate.
