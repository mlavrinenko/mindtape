<!-- Generated from docs/src/agents.typ by `just docs md`. Do not edit; edit the source. -->

# MindTape — AI Agent Context

@CONTRIBUTING.md says how work is done in this repo. This file says what MindTape is and where its parts live.

## What is this?

MindTape is a file-based task tracker. Each task is one Typst file (`#show: task.with(...)`), and the filename stem is its identity. Every `mt` command evaluates the task files under the configured roots through the Typst compiler crates. There is no daemon and no database. A content-hash cache in `.mindtape/cache/` skips evaluating a file whose bytes did not change.

## Repo map

- `lib/prelude.typ` — the `@local/mindtape` Typst package, which defines the task DSL.
- `truth/tasks/` — MindTape’s own backlog. `mt` here uses the in-tree `lib/`.
- `truth/goals.typ` and `truth/process-goals.typ` — the product and process goals. Each open task rates every goal in its `x-goals` field, and the `goals` rule checks the ratings with `check-goals` from `truth/lib.typ`.
- `www/content/*.typ` — the docs site, built with tola.
- `docs/src/*.typ` — the sources of README.md, AGENTS.md, CONTRIBUTING.md, the skill’s SKILL.md and the `docs/contributing/*.md` deep dives.
- `skills/mindtape/` — the agent skill for driving `mt`, with its references and evals.
- `examples/` — fixture projects. `learn-piano` uses the full config surface and feeds the docs and the skill’s evals. `minimal` is the smallest config. `dsl` holds the task the DSL block below quotes. `large` is a 2000-task benchmark whose gitignored tasks `just gen-bench` writes. Each of the others backs one docs page.

Before you write about a feature, read the `www/content/` page that covers it: `dsl.typ` for task fields, edge kinds and `recur`, `templates.typ` for `mt add` and slots, `derived-tags.typ`, `priority.typ`, `filters.typ`, and `config.typ` for every config key.

## Workspace

The cargo workspace has a virtual root manifest and two crates:

- `crates/mindtape` — the domain library: config, task model, filter language, validation and project loader. Other tools, such as speconaut, depend on it.
- `crates/mindtape-cli` — the thin `mt` binary, built on clap.

The Typst plumbing comes from seven crates.io dependencies, and this repo holds none of their source:

- `artifact-gate` — the rule engine behind `mt check`’s body header rules
- `swhid-mint` — mints the SWHID a tombstone carries
- `typst-cleanup` — retires a tracked file: dirty-tree guard, tombstone, link rewriting
- `typst-cst` — reads a file’s heading tree by parsing, without evaluating
- `typst-edit` — rewrites a file in place at known spans
- `typst-harvest` — evaluates a file and collects the data it marks for export
- `typst-world` — the eval-only Typst World the others share

## Architecture

- Rust, with the toolchain pinned in `rust-toolchain.toml`.
- Typst runs eval-only through `typst-eval`. MindTape never lays out or renders a document.
- `evalexpr` parses the filter language. MindTape walks the parsed tree and evaluates it itself.
- `clap` for the CLI, `serde` and `toml` for config, `serde_json` and `csv` for output, and a hand-rolled table renderer in `crates/mindtape-cli/src/render/`.

## Task DSL (lib/prelude.typ)

```typ
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Fix the login redirect",
  status: wip(2026, 6, 14)[reproduced on staging], // the status, the date it was set, and a note
  priority: ice(impact: 8, confidence: 0.7, ease: 6), // a framework call or a number; difficulty too
  estimate: (days: 1, hours: 2), // units the project allows
  assignee: "ada", // who is working on it
  links: depends-on("add-a-session-store.typ"), // typed edges; a target is a path from this file
  tags: ("backend",),
  x-speconaut: (spec: "login-flow"), // a tool's own field; mt passes it through
)
```

`task` declares each core field as a named parameter, so tinymist completes the field names. Its `..meta` sink takes only `x-<ns>` extension slots, and a misspelled field such as `titel` fails the file with an `unknown task slot` error.

A field value is a plain shape that Rust interprets. `ice(...)` and the other framework helpers build a `(framework, inputs)` dict, `every(...)` and `rrule(...)` build a recurrence dict, and `depends-on("a.typ") + related("b.typ")[why]` builds typed edges. Rust does all math and date arithmetic.

`docs/contributing/validation-seam.md` says which layer a new validation belongs in: the prelude, Rust and config, or the card render.

### Mechanism, not policy

Rust ships mechanisms, and projects write policy as config expressions. Rust code never names a specific estimate unit, status, tag, path or threshold. A new check is a `[[tasks.rules]]` entry. When the filter language cannot express it, extend the language with a field, predicate, aggregate or git state instead of adding a config table or checker of its own. Shipped defaults live in rule packs under `crates/mindtape/src/packs/`, and a project overrides a pack rule by name.
