<!-- Generated from docs/src/agents.typ by `just docs-md`. Do not edit; edit the source. -->

<h1>MindTape — AI Agent Context</h1>

How to contribute — quality gates, testing, file-size caps, commit rules, the signal sweep, dependency drift, version bumps — lives in @CONTRIBUTING.md. That reference pulls the whole file into your context automatically, so nothing here restates any of it. If you are about to write a rule about _how work is done_, it belongs there, not here.

This file carries the other half: what MindTape _is_.

## What is this?

A daemon-free, file-based task tracker. Tasks are one-per-file Typst artifacts (`#show: task.with(...)`). MindTape live-evaluates the configured roots on every command via the Typst compiler crates — no watch daemon, no SQLite, no global index. Identity is the filename stem.

Repository: [https://github.com/mlavrinenko/mindtape](https://github.com/mlavrinenko/mindtape)

## Repo map

- `lib/prelude.typ` — the `@local/mindtape` Typst package (the task DSL)
- `truth/tasks/` — MindTape’s own backlog as per-file task artifacts (dogfood); `mt ls` / `mt check` run against it via the in-tree `lib/`. Decisions live here too — the ADR process is retired
- `www/content/*.typ` — the docs site (tola). Reference material lives here, not in this file; read the page rather than restating it
- `docs/contributing/*.md` — contributor deep dives behind CONTRIBUTING.md
- `examples/` — `learn-piano` (full config surface, feeds the docs generator and the skill’s evals) and `minimal` (the floor config)

Before writing about a feature, check whether the answer is already one of:

| Topic | Read |
| --- | --- |
| Every task field, `links:` edge kinds, `recur` | `www/content/dsl.typ` |
| `mt add`, slots, `--set`, `--describe`, `--edit` | `www/content/templates.typ` |
| Derived tags from folder layout | `www/content/derived-tags.typ` |
| Priority frameworks, scoring, bands | `www/content/priority.typ` |
| Filter language | `www/content/filters.typ` |
| Every config key | `www/content/config.typ` |
| Driving `mt` as an agent | `skills/mindtape/SKILL.md` |

## Workspace

2 crates (`cargo` workspace, virtual root manifest):

- `crates/mindtape` — domain library: config, task model, filter language, validation, project loader. Other tools (e.g. speconaut) depend on this.
- `crates/mindtape-cli` — the thin `mt` binary (clap).

The former in-tree substrate crates — 7, not three — are published to crates.io as regular registry dependencies:

- `artifact-gate` `0.1.0` — artifact validation
- `swhid-mint` `0.1.1` — SWHID minting
- `typst-cleanup` `0.2.0` — git-aware cleanup
- `typst-cst` `0.2.0` — parse-plane structure extraction
- `typst-edit` `0.1.0` — slot/link rewriting
- `typst-harvest` `0.3.0` — harvest
- `typst-world` `0.3.0` (transitive) — package-agnostic Typst World/eval

The crate marked transitive above is reached only through another substrate crate, never named directly in a workspace manifest — `mindtape` does not depend on it directly.

Name, version, and direct-vs-transitive are derived, never hand-listed — see `www/generated/crate-map.json` (`www/generate-facts.sh`, consumed by `www/content/internals.typ`).

## Architecture

- Rust (edition 2024, pinned `rust-toolchain.toml`)
- Typst 0.15 eval-only (`typst-eval`); never layout or render
- `evalexpr` 12 as a parser for the filter language (walked, not evaluated by it)
- `clap` 4 CLI; `serde`/`toml` config; a hand-rolled table renderer (`crates/mindtape-cli/src/render/`), `serde_json`/`csv` output

The dev shell and its tooling are `flake.nix`’s subject, and the workflow around each tool is CONTRIBUTING.md’s — neither is restated here.

## Task DSL (lib/prelude.typ)

```typ
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Title",
  status: wip(2026, 6, 14)[note],    // (name, date, note); note is a trailing block
  priority: ice(impact: 8, confidence: 0.7, ease: 6), // or a raw int; difficulty too
  estimate: (days: 1, hours: 2),     // opaque unit dict; allowed units per project
  assignee: "ada",                   // actor working it; config-bounded, not ownership
  links: depends-on("other.typ"),    // typed edges by file stem
  tags: ("backend",),
  x-speconaut: ...,                  // opaque extension slot (x-<ns>)
)
```

`task` declares every core field as a named parameter (`..meta` is the `x-<ns>` extension sink only), so tinymist completes the field names and shows the whole signature; a misspelled field reaches the sink and fails the file with an `unknown task slot` error naming the field, e.g. `titel`.

Values are shapes, never computed results: `ice(...)` and the other framework packagers build a `(framework, inputs)` dict, `every(...)`/`rrule(...)` build a recurrence dict, `depends-on("a.typ") + related("b.typ")[why]` builds typed edges. No math and no date arithmetic happen in Typst — the Rust layer does all of it.

### Where validation lives

The seam that decides which layer a change belongs in:

- The prelude enforces structure only — one task, the `x-<ns>` discipline, and the argument shapes it would otherwise drop before the marker is written (edge targets and notes, a status note, framework inputs). Those are the mistakes the Rust layer can never see, because nothing reaches it to validate.
- Everything that DOES reach the marker is the Rust layer’s: the status legend, the numeric priority/difficulty scales and classification, priority frameworks (scoring, in-bounds by construction), the edge-kind enum and its per-kind rules, the `required` rules (a filter expression per field), bounds, and link resolution.
- Consumers validate their own `x-<ns>` fields. MindTape carries them through untouched.

So: a new constraint on a value goes in Rust and config. A new constraint on the _call shape_ goes in the prelude. Putting either in the other place produces a rule that cannot fire.
