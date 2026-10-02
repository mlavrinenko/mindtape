#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write the config schema descriptions for users",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt", "docs"),
  estimate: (src-files: 5),
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
      docs-match-code: helps[the /config/ table and editor hovers explain keys in user terms],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
)

== Problem

+ Run `rg -c 'crate::' schema/config.schema.json`.
+ 🔴 It prints `5`. One description reads `` [`crate::config_packs`] — so this field always reads back as whichever layer ... ``.
+ Editors show these descriptions on hover over `.mindtape/config.toml`, and the /config/ Sections table prints them through `www/utils/config-ref.typ`. Both surfaces show Rust intra-doc links to users.


== TO-BE

- 🟢 Each description says what the key does and which values it takes, in user terms.
- 🟢 Notes for Rust developers move to `//` comments, which the schema does not export.
- 🟢 A test fails on a schema description containing `crate::` or a Rust intra-doc link, naming the key.


== Changes

- `crates/mindtape/tests/schema_descriptions.rs`, new: the test. It fails on today's schema first.
- `crates/mindtape/src/config*.rs`: rewrite the doc comments the test names.
- `schema/config.schema.json`: regenerated.
- `www/utils/config-ref.typ`: render descriptions as markup, so backticks become code.


== Rollback

Revert the commit; the schema text goes back to the old doc comments.


== Out of scope

- The schema's structure and key names: wording only.


== Dogfood

This repo's `.mindtape/config.toml` gets the new hover text in any editor with a TOML language server.

