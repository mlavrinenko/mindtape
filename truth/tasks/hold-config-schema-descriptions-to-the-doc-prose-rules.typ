#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse history words in config schema descriptions",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 3,
  estimate: (src-files: 3),
  tags: ("found", "docs"),
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
      docs-match-code: helps[the schema text the site and editors show is held to the history rule hand-written docs are],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== Problem

+ Rendering config key tables from the schema for /statuses/, /rules/ and /links/
+ 🔴 #raw("schema/config.schema.json"): config doc comments reach the docs site and editors through the schema, and no check refuses a history word in them.

`crates/mindtape/tests/schema_descriptions.rs` refuses a `crate::` path and a rustdoc intra-doc link, and nothing else. `758441b6` removed "Defaults to `local-task-path`, which is exactly the historical behaviour" from `LinkKind.validate` by hand, so the wording reaches the schema and nothing stops it coming back.

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/check-doc-prose.sh").

== TO-BE

- 🟢 `just check-doc-prose` fails on a history word in a `description` of `schema/config.schema.json`, naming the key, with the same word list it applies to the doc sources.

== Changes

- `.just/scripts/check-doc-prose.sh`: read every `description` out of the schema and scan it with the existing word lists.
- `.mmz/config.yaml`: add `policy`, which holds `schema/**`, to the check-doc-prose rule's inputs.
- `Justfile`: say in the recipe comment that the schema descriptions are scanned.

== Rollback

Revert the script change.

== Out of scope

- Six descriptions the scan flagged: reworded in `reword-the-six-config-schema-descriptions-that-read-as-history`.
- Rust paths and intra-doc links: `crates/mindtape/tests/schema_descriptions.rs` already refuses them.
- Rustdoc on internal items that never reach the schema: no reader outside the crate sees it.

== Dogfood

The /config/, /statuses/, /rules/, /links/ and /cli/ key tables render these descriptions on every docs build.
