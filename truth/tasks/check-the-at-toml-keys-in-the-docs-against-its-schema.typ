#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check the at.toml keys in the docs against its schema",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("docs", "www", "tooling"),
  estimate: (src-files: 3),
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
      docs-match-code: helps[a misspelled or renamed at.toml key in a page fails the span gate, as a config key does],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 17),
  links: parent("keep-the-user-docs-generated-or-gated-not-restated.typ")
    + depends-on(
      "derive-a-json-schema-for-at-toml-from-its-structs.typ",
    )[the gate and the key table read schema/at.schema.json, which that task writes],
)

== Problem

+ Add a sentence with the spans `[board] sortt` and `[ls] hyperlinkz` to `www/content/read-many-projects.typ` and run `just check-doc-spans`.
+ 🔴 It flags only the second, printing `ls has no key hyperlinkz; closest: hyperlinks`. `[board] sortt` passes, because `rooted` in `.just/scripts/check-doc-spans.jq` checks a span only when its first segment is a top-level key of `schema/config.schema.json`.
+ 🔴 The page lists `include`, `exclude` and `forks` in prose, and `www/content/config.typ` names `at.toml` only in its environment-variable table.


== TO-BE

- 🟢 A span whose first segment is a top-level key of `schema/at.schema.json`, such as `[board] sortt`, resolves against that schema and fails with the closest key.
- 🟢 The page renders the `at.toml` keys with `config-keys-table` from that schema, `fallback` included, in place of the prose list.
- 🟢 `config.typ` links that table where it names `at.toml`.


== Source of truth

- `schema/at.schema.json`, which the schema task derives from `crates/mindtape/src/at/`.


== Changes

- `.just/scripts/check-doc-spans.sh`: the red check first: the control page gains `[board] sortt`, which must fail. Pass `schema/at.schema.json` to the jq program.
- `.just/scripts/check-doc-spans.jq`: resolve a span in the schema that holds its first segment.
- `www/generate.sh`: copy `schema/at.schema.json` to `www/generated/at-schema.json`.
- `www/content/read-many-projects.typ`: the key table under "Name your projects".
- `www/content/config.typ`: the link.


== Out of scope

- The shipped board's values, which a sibling task shows from `board.toml`.


== Dogfood

This repo's `just check` runs the span gate over the read-many-projects page, the one page that names `at.toml` keys.

