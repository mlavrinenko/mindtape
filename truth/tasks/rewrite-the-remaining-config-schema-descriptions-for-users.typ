#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the remaining config schema descriptions for users",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt", "docs"),
  estimate: (src-files: 14),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[editor hovers state what each config key does, in user terms],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
  links: parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Run `jq -r '."$defs".LinkKind.properties.validate.description' schema/config.schema.json`.
+ 🔴 Its second line reads `which is exactly the historical behaviour (a stem resolved against the`.
+ Other descriptions name Rust terms (`None`, `Vec<RootSpec>`, the `uuid` crate, evalexpr) or use figures (`policy floor`, `the deliberate opt-out`, `white-list`).
+ Two state behaviour `mt` does not have. `[[tasks.status]] note` says `forbidden` is enforced, and `terminal` says required fields relax.


== TO-BE

- 🟢 Every description in `schema/config.schema.json` says what the key does and which values it takes, in user terms.
- 🟢 Each description matches what `mt` does.


== Changes

- `crates/mindtape/src/config_status.rs`: `terminal`, `to`, `note`, `unblocked` and the legend.
- `crates/mindtape/src/link.rs`: `[[tasks.links.kinds]]` keys.
- `crates/mindtape/src/config_links_validate.rs`: the `validate` values.
- `crates/mindtape/src/config_roots.rs`: root and tag derivation keys.
- `crates/mindtape/src/framework.rs`: framework `score`, `category` and inputs.
- `crates/mindtape/src/config_hooks.rs`: `[[on.flip]]` actions.
- `crates/mindtape/src/config_ls_columns.rs`: `[ls.columns.<name>] when`.
- `crates/mindtape/src/config_schema.rs`: the root description and the hand-written schemas.
- `crates/mindtape/src/body_headers.rs`, `config_rm.rs`, `config_go.rs`, `config_actors.rs`, `recur.rs`: wording.
- `schema/config.schema.json`: regenerated.


== Rollback

Revert the commit; the schema text goes back to the old doc comments.


== Out of scope

- Key names and schema structure: wording only.
- A new test. The `schema_descriptions` test already fails on Rust paths and intra-doc links; plain wording has no mechanical check.


== Dogfood

This repo's `.mindtape/config.toml` gets the new hover text in any editor with a TOML language server.

