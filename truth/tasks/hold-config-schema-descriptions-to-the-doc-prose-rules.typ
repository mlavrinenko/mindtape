#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Hold config schema descriptions to the doc prose rules",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
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
      docs-match-code: helps[the schema text the site and editors show follows the same rules as hand-written docs],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 3),
)

== Problem

+ Rendering config key tables from the schema for /statuses/, /rules/ and /links/
+ 🔴 #raw("schema/config.schema.json"): Config doc comments reach the docs site and editors through the schema, and some carry rustdoc intra-doc links and history that no gate checks.

`LinkKind.validate` reads "Defaults to `local-task-path`, which is exactly the historical behaviour". `NamingConfig.dedup` reads "see [`NamingConfig::effective_dedup`]". `extends` reads "see [`crate::config_packs`]" in the /config/ section map.

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/check-doc-prose.sh").

== TO-BE

- 🟢 `just check-doc-prose` fails on a history word or a rustdoc intra-doc link in a `description` of `schema/config.schema.json`, naming the key.

== Changes

- `.just/scripts/check-doc-prose.sh`: scan the schema descriptions.
- `crates/mindtape/src/config*.rs`: reword the doc comments it flags, then re-bless the schema.

== Rollback

Revert the script change. The reworded doc comments can stay.

== Out of scope

- Rustdoc on internal items that never reach the schema: no reader outside the crate sees it.

== Dogfood

The /config/, /statuses/, /rules/, /links/ and /cli/ key tables render these descriptions on every docs build.
