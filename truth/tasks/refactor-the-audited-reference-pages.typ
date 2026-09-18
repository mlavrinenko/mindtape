#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refactor the audited reference pages onto their topics",
  kind: "docs",
  status: done(
    2026,
    9,
    19,
  )[nine pages refactored; schema owns the applied contract and names the recur gap; audit down to two accepted pairs],
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (files: 10),
  assignee: "agent",
  x-goals: (
    product: (
      readable-tasks: helps[a page stating a fact the code contradicts teaches the wrong thing],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the JSON schema page matches canonical.rs and the ICE formula it prints],
    ),
  ),
)

== Problem

1. 🔴 The audit of the reference-side pages found facts the code contradicts:
  `named-sets` says a view's `filter` is required (it defaults to match-all),
  `schema` shows a `"score": 6.72` the ICE formula cannot produce and sketches
  the `applied` contract `/agents/` delegates to it, `schema` and
  `import-export` call `mt ls --json` lossless while canonical JSON carries no
  `recur`, `rule-packs` counts six sections where the pack ships five, and
  `internals` renders a transitive-crate clause that never fires because all
  seven crates are direct.

== TO-BE

- 🟢 Every page matches the sources and states its part once.
- 🟢 `schema` fills the `applied` contract `agents.typ` points at, and
  `agents.typ`'s `truncated` copy becomes a link to it.
- 🟢 `rule-packs` cross-links `/config/` for the layer merge; `named-sets` owns
  the dictionary counts; `hooks` loses its jargon; `internals` links `/hooks/`
  and `/quickstart/`; `self-validating-tasks` says the no-linter point once.

== Source of truth

- `crates/mindtape/src/canonical.rs`, `import_map.rs`, `frameworks.toml`,
  `config_ls.rs`, `config_tasks.rs`, `crates/mindtape-cli/src/output.rs`, and
  `www/generated/crate-map.json`.

== Changes

- The seven pages, `config.typ` (drop the rule-pack restatement), `agents.typ`
  (the `truncated` pointer).

== Out of scope

- Slicing fixture config halves and deriving the built-in tables; those are
  filed separately.

== Dogfood

`tola query` per page, `just dup-prose` before and after, `just docs::check`.
