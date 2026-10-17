#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse an em dash in the site text, the schema descriptions and the help",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 2,
  tags: ("docs", "www", "tooling"),
  estimate: (src-files: 35),
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
      docs-match-code: helps[the text the docs build generates keeps the voice the hand-written pages keep, under a gate],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17),
  links: parent("keep-the-user-docs-generated-or-gated-not-restated.typ")
    + depends-on(
      "name-the-slug-column-in-the-hyperlinks-help-and-offer-it-only-where-it-applies.typ",
    )[both rewrite the \[ls\] hyperlinks doc comments in config/ls.rs and regenerate schema/config.schema.json],
)

== Problem

+ Run `mt config schema | jq '[.. | objects | .description? | strings | select(test("—"))] | length'`.
+ 🔴 It prints `59`. Descriptions such as "`[add]` — interactive editor fill settings." reach the `/config/` Sections table and `mt config schema` in a register the site's own pages don't use.
+ 🔴 `www/utils/` puts 8 more on the site, such as the `mindtape` crate note "The domain library — config, the task model, …". `.just/scripts/check-doc-prose.sh` reads every schema description and has no em-dash rule.


== TO-BE

- 🟢 `check-doc-prose` fails on an em dash in a site source, in rendered `www/utils/` text, in a schema description and in a `cli.json` help string, naming the file and line.
- 🟢 The 59 descriptions read like the site: one idea per sentence, starting from what the key does for the reader.
- 🟢 `docs/contributing/documentation.md` lists the rule beside the others `check-doc-prose` enforces.


== Source of truth

- `docs/contributing/documentation.md`: the Voice and Plain prose rules.
- `mt config schema` and `mt _cli --json`: the generated text.


== Changes

- `.just/scripts/check-doc-prose.sh`, `.just/scripts/check-doc-prose.awk`: the red check first: the em-dash rule, failing on today's tree. Then read `cli.json` help strings and `schema/at.schema.json` descriptions too.
- `crates/mindtape/src/config/*.rs` and the other modules whose doc comments feed the schema, about 30 files: rewrite each description.
- `schema/config.schema.json`: regenerate.
- `www/utils/crate-notes.typ`, `www/utils/sweep-flag-notes.typ`, `www/utils/cli-ref.typ`, `www/utils/slices.typ`: rewrite the rendered ones.
- `docs/src/contributing/documentation.typ`: the rule.


== Out of scope

- AGENTS.md, the skill and code comments, whose readers aren't the site's.
- Build error messages in `www/utils/page.typ` and `site.typ`, which only a contributor reads.


== Dogfood

This repo's `just check` runs the rule, and its own `/config/` page is the first one it cleans.

