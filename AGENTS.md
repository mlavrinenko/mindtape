<!-- Generated from docs/src/agents.typ by `just docs md`. Do not edit; edit the source. -->

# MindTape — AI Agent Context

How work is done lives in @CONTRIBUTING.md, pulled into your context whole by that reference; a process rule belongs there and is never restated here. This file carries the other half: what MindTape _is_.

## What is this?

A daemon-free, file-based task tracker. Tasks are one-per-file Typst artifacts (`#show: task.with(...)`). MindTape live-evaluates the configured roots on every command via the Typst compiler crates — no SQLite, no global index. Identity is the filename stem.

Repository: [https://github.com/mlavrinenko/mindtape](https://github.com/mlavrinenko/mindtape)

## Repo map

- `lib/prelude.typ` — the `@local/mindtape` Typst package (the task DSL)
- `truth/tasks/` — MindTape’s own backlog as per-file task artifacts (dogfood); `mt ls` / `mt check` run against it via the in-tree `lib/`
- `truth/goals.typ` and `truth/process-goals.typ` — what MindTape is for, and how this repo builds it; every open task gives each goal a verdict in its `x-goals` field, and the `goals` rule checks them with `check-goals` from `truth/lib.typ`
- `www/content/*.typ` — the docs site (tola)
- `docs/contributing/*.md` — contributor deep dives behind CONTRIBUTING.md
- `examples/` — `learn-piano` (full config surface; feeds the docs generator and the skill’s evals), `minimal` (the floor config), and `large` (a 2000-task benchmark fixture; its tasks are gitignored, `just gen-bench` rebuilds them)

Reference material lives on the docs site, not here. Before writing about a feature, read whichever page below already answers it rather than restating it — the page at `/<stem>/` is `www/content/<stem>.typ`.

| Topic | Read |
| --- | --- |
| Every task field, `links:` edge kinds, `recur` | [Task DSL](https://mlavrinenko.github.io/mindtape/dsl/) |
| `mt add`, slots, `--set`, `--describe`, `--edit` | [Templates & slots](https://mlavrinenko.github.io/mindtape/templates/) |
| Derived tags from folder layout | [Derived tags](https://mlavrinenko.github.io/mindtape/derived-tags/) |
| Priority frameworks, scoring, bands | [Priority](https://mlavrinenko.github.io/mindtape/priority/) |
| Filter language | [Filter language](https://mlavrinenko.github.io/mindtape/filters/) |
| Every config key | [Configuration](https://mlavrinenko.github.io/mindtape/config/) |
| Driving `mt` as an agent | `skills/mindtape/SKILL.md` |

## Workspace

Two crates (`cargo` workspace, virtual root manifest):

- `crates/mindtape` — domain library: config, task model, filter language, validation, project loader. Other tools (e.g. speconaut) depend on this.
- `crates/mindtape-cli` — the thin `mt` binary (clap).

The seven former in-tree substrate crates are published to crates.io and consumed back as regular registry dependencies:

- `artifact-gate` `0.1.0` — artifact validation
- `swhid-mint` `0.1.1` — SWHID minting
- `typst-cleanup` `0.2.0` — git-aware cleanup
- `typst-cst` `0.2.0` — parse-plane structure extraction
- `typst-edit` `0.1.0` — slot/link rewriting
- `typst-harvest` `0.3.0` — harvest
- `typst-world` `0.3.2` — package-agnostic Typst World/eval

Both lists — name, version, direct-vs-transitive — are derived from `www/generated/crate-map.json`, never hand-written.

## Architecture

- Rust (edition 2024, pinned `rust-toolchain.toml`)
- Typst 0.15 eval-only (`typst-eval`); never layout or render
- `evalexpr` 12 as a parser for the filter language (walked, not evaluated by it)
- `clap` 4 CLI; `serde`/`toml` config; a hand-rolled table renderer (`crates/mindtape-cli/src/render/`), `serde_json`/`csv` output

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

- The prelude enforces structure only — one task, the `x-<ns>` discipline, and the argument shapes it rejects before the marker is written (edge targets and notes, a status note, framework inputs). Those are the mistakes the Rust layer can never see, because nothing reaches it to validate.
- Everything that DOES reach the marker is the Rust layer’s: the status legend, the numeric priority/difficulty scales and classification, priority frameworks (scoring, in-bounds by construction), the edge-kind enum and its per-kind rules, the `required` rules (a filter expression per field), bounds, and link resolution. Types too — `crates/mindtape/src/marker.rs` reports a wrong-shaped field, and any wrong-typed element inside a well-shaped one, naming the position (`tags[1]`, `estimate.days`, `recur.on[1]`).
- The card render enforces nothing. It is total by construction — every accessor type-guards, `_scalar` falls back to `repr` — so a wrong-typed value reaches the marker instead of failing eval on a method it does not have. An eval error there names neither the field nor the task, and disappears the moment the render stops making that call.
- Consumers validate their own `x-<ns>` fields. MindTape carries them through untouched.

So: a new constraint on a value goes in Rust and config. A new constraint on the _call shape_ goes in the prelude. Putting either in the other place produces a rule that cannot fire.

### Mechanism, not policy

Rust ships mechanisms; projects write policy as config expressions. Rust never names a specific estimate unit, status, tag, path or threshold. A new check is a `[[tasks.rules]]` entry, and when the filter language cannot express it, extend the language (a field, a predicate, an aggregate, a git state) rather than add a bespoke config table or checker. Shipped defaults live in a rule pack (`crates/mindtape/src/packs/`), where a project overrides them by name.
