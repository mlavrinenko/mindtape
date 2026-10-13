<!-- Generated from docs/src/agents.typ by `just docs md`. Do not edit; edit the source. -->

# Mindtape — AI Agent Context

@CONTRIBUTING.md says how work is done here. This file says what Mindtape is and where its parts live. Fork parallel work with `just fork` and land it with `just land`.

## What is this?

Mindtape is a file-based task tracker. Each task is one Typst file (`#show: task.with(...)`), named by its filename stem. Every `mt` command evaluates the task files under the configured roots through the Typst compiler crates, with no daemon or database. A content-hash cache in `.mindtape/cache/` skips a file whose bytes are unchanged.

## Repo map

- `lib/prelude.typ` — the `@local/mindtape` package, which defines the task DSL.
- `truth/tasks/` — Mindtape’s own backlog, read with the in-tree `lib/`. Change a task with `mt` commands, as `skills/mindtape/SKILL.md` says. Edit the file by hand only for the body, a framework priority, `estimate` and `x-goals`, then run `mt check`.
- `truth/goals.typ` and `truth/process-goals.typ` — the product and process goals. Each open task rates every goal in `x-goals`, which the `goals` rule checks.
- `www/content/*.typ` — the docs site, built with tola.
- `docs/src/*.typ` — the sources of README.md, AGENTS.md, CONTRIBUTING.md, SKILL.md and the `docs/contributing/*.md` deep dives.
- `skills/mindtape/` — the agent skill for driving `mt`, with its references and evals.
- `examples/` — fixture projects: `api-service`, a sample backend backlog that feeds the docs and the skill’s evals; `minimal`, the smallest config; `dsl`, the task quoted below; `large`, a 2000-task benchmark whose gitignored tasks `just gen-bench` writes. Each other one backs one docs page.

Before you write about a feature, read the `www/content/` page that covers it: `dsl.typ` for task fields, edge kinds and `recur`, `templates.typ` for `mt add` and slots, `derived-tags.typ`, `priority.typ`, `filters.typ`, and `config.typ` for every config key.

## Workspace

The cargo workspace has a virtual root manifest and two crates:

- `crates/mindtape` — the domain library: config, task model, filter language, validation and project loader, which speconaut and other tools depend on.
- `crates/mindtape-cli` — the thin `mt` binary, built on clap.

The Typst plumbing comes from seven crates.io dependencies, whose source sits under `~/.cargo/registry/src/` at the versions `Cargo.lock` pins. `HVal`, the value every task field passes through, is `typst_world::value::HVal`.

- `artifact-gate` — the rule engine behind `mt check`’s body header rules
- `swhid-mint` — mints the SWHID a tombstone carries
- `typst-cleanup` — retires a tracked file: dirty-tree guard, tombstone, link rewriting
- `typst-cst` — reads a file’s heading tree by parsing, without evaluating
- `typst-edit` — rewrites a file in place at known spans
- `typst-harvest` — evaluates a file and collects the data it marks for export
- `typst-world` — the eval-only Typst World the others share

## Architecture

- Typst runs eval-only through `typst-eval`: Mindtape never lays out a document.
- `evalexpr` parses the filter language, and Mindtape evaluates the parsed tree itself.
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

`task` declares each core field as a named parameter, and its `..meta` sink takes only `x-<ns>` slots, so a misspelled field fails.

A field value is a plain shape that Rust interprets. `ice(...)` and the other framework helpers build a `(framework, inputs)` dict, `every(...)` and `rrule(...)` build a recurrence dict, and `depends-on("a.typ") + related("b.typ")[why]` builds typed edges. Rust does all math and date arithmetic.

### Mechanism, not policy

Rust ships mechanisms, and projects write policy as config expressions. Name no estimate unit, status, tag, path or threshold in Rust code. Write a new check as a `[[tasks.rules]]` entry. When the filter language cannot express it, extend the language with a field, predicate, aggregate or git state, never a config table or checker of its own. Put shipped defaults in the rule packs under `crates/mindtape/src/packs/`, where a project overrides a rule by name. `docs/contributing/validation-seam.md` says which layer a new check belongs in.
