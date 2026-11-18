<!-- Generated from docs/src/agents.typ by `just docs md`. Do not edit; edit the source. -->

# Mindtape: AI Agent Context

@CONTRIBUTING.md says how work happens here. This file says what Mindtape is and where to find each part. Fork parallel work with `just fork` and land it with `just land`.

## Overview

Mindtape is a file-based task tracker. Each task is one Typst file (`#show: task.with(...)`), named by its filename stem. Every `mt` command evaluates the task files under its roots through the Typst compiler crates, with no daemon or database. A content-hash cache in `.mindtape/cache/` skips unchanged files.

## Repo map

- `lib/prelude.typ`: the `@local/mindtape` package, which defines the task DSL.
- `crates/mindtape/src/loader/preludes/`: an immutable snapshot of each prelude version this binary reads, `0.2.0`, `0.3.0` and `0.4.0`.
- `truth/tasks/`: Mindtape’s own backlog, a folder per month plus a `swept/` inbox, read with the in-tree `lib/`.
- `truth/goals.typ` and `truth/process-goals.typ`: the product and process goals. Each open task rates every goal in `x-goals`. The `give-every-goal-verdict` rule checks that.
- `www/content/*.typ`: the docs site, built with tola.
- `docs/src/*.typ`: the sources of README.md, AGENTS.md, CONTRIBUTING.md, SKILL.md and the `docs/contributing/*.md` deep dives.
- `skills/mindtape/`: the agent skill for driving `mt`, with its references and evals.
- `examples/`: fixture projects backing one docs page each.
  - `api-service`: a sample backend backlog that feeds the docs and the skill’s evals.
  - `minimal`: the smallest config.
  - `dsl`: the task quoted on the DSL page.
  - `large`: a 2000-task benchmark whose gitignored tasks `just bench gen` writes.
  - `nix-adopter`: a flake that takes `mt` into its dev shell, which `just smoke-flake` locks against the working tree.

Before writing about a feature, read its `www/content/` page: `dsl.typ` for task fields, edge kinds and `recur`, `templates.typ` for `mt add` and slots, `hooks.typ` for hooks, `derived-tags.typ`, `priority.typ`, `filters.typ`, and `config.typ` for every config key.

## Workspace

The cargo workspace has a virtual root manifest and two crates:

- `crates/mindtape`: the domain library: config, task model, filter language, validation and project loader, which speconaut and other tools depend on.
- `crates/mindtape-cli`: the thin `mt` binary, built on clap.

The Typst plumbing comes from seven crates.io dependencies, under `~/.cargo/registry/src/` at the versions `Cargo.lock` pins. `HVal`, the value every task field passes through, is `typst_world::value::HVal`.

- `artifact-gate`: the rule engine behind `mt check`’s body header rules.
- `swhid-mint`: mints the SWHID a tombstone records.
- `typst-cleanup`: retires a tracked file with a dirty-tree guard, a tombstone, and link rewriting.
- `typst-cst`: reads a file’s heading tree by parsing, without evaluating.
- `typst-edit`: rewrites a file in place at known spans.
- `typst-harvest`: evaluates a file and collects the data it marks for export.
- `typst-world`: the eval-only Typst World the others share.

## Architecture

- Typst runs eval-only through `typst-eval`: Mindtape never lays out a document.
- `evalexpr` parses the filter language, and Mindtape evaluates the parsed tree itself.
- `clap` for the CLI, `serde` and `toml` for config, `serde_json` and `csv` for output, and a hand-rolled table renderer in `crates/mindtape-cli/src/render/`.

`schema/output/` holds the committed schema of every `--json` output, generated from the Rust types. When an output’s structure changes, bump its version and run `mt schema --write`. `mt schema --check` runs in the gates.

## Task definition language (lib/prelude.typ)

`mt add` writes a task file, and `mt` commands change it, as `skills/mindtape/SKILL.md` says. Run `mt explain` for the shape this repo declares, `mt explain templates` for the templates `mt add` files from, and `mt explain rules --for <moment>` for what an add, commit or flip checks. Edit a task by hand only for the body, a framework priority, `estimate` and `x-goals`, then run `mt check`.

### Mechanism, not policy

Rust provides mechanisms, and projects write policy as config. Rust code doesn’t state an estimate unit, status, tag, path or threshold. Write a new check as a `[[tasks.rules]]` entry. When the filter language cannot express it, extend the language with a field, predicate, aggregate or git state, never a config table or checker of its own. Put built-in defaults in the rule packs under `crates/mindtape/src/packs/`, where a project overrides a rule by name. `docs/contributing/validation-seam.md` says which layer a new check belongs in.
