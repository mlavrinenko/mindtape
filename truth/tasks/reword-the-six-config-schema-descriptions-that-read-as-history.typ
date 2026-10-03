#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Reword the six config schema descriptions that read as history",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("found", "docs"),
  estimate: (src-files: 6),
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
      docs-match-code: helps[the descriptions state what a key does, in words the plain-prose check accepts],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== Problem

+ `jq -r '.["$defs"].RuleSpec.properties.name.description' schema/config.schema.json`
+ 🔴 It prints "What the rule is called, used to report it (`rules.<name>`)". The `name` descriptions of `CommitRuleSpec`, `ContentRuleConfig` and `HeaderRuleConfig` open the same way, and the doc prose check refuses "used to" as history. The `ready` description of `LsConfig` and the description of `ReadyConfig` write the column header `READY` in capitals.

== TO-BE

- 🟢 The four rule `name` descriptions say "The name a report shows for the rule".
- 🟢 The two `ready` descriptions write `READY` as code.


== Changes

- `crates/mindtape/src/{config_commits,config_rules,body_headers,body_content,config_ls}.rs`: reword the six doc comments.
- `schema/config.schema.json`: re-bless with `MINDTAPE_BLESS=1 cargo test -p mindtape schema_matches_golden`.


== Rollback

Revert the commit.

== Out of scope

- The check itself: `hold-config-schema-descriptions-to-the-doc-prose-rules` adds it.


== Dogfood

The key tables on /config/, /statuses/, /rules/ and /cli/ render these descriptions.
