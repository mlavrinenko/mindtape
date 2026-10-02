#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Split the config page into config, statuses and rules pages",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("www", "docs"),
  estimate: (src-files: 2),
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
      docs-match-code: helps[each config topic has one page, and its TOML comes from a checked fixture],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
  links: depends-on(
    "write-plain-prose-rules-for-the-docs-and-check-for-history-and-shouting.typ",
    "refuse-hand-typed-code-blocks-in-doc-sources.typ",
  )
    + parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Open /config/.
+ 🔴 The source is 827 lines with about 20 top-level sections: the layer merge, the status legend, rules, link kinds, environment variables, local packages.
+ 🔴 `[[ls.rules]]` is described in `cli.typ:163`, `config.typ:664` and `filters.typ:364`. `config.typ:130` says the CLI page owns it, and `cli.typ:170` says the config page does.
+ 🔴 `config.typ:736` shows `kind = "spike"` under `[[overlay.tasks.links.kinds]]`; loading it prints `unknown field kind`. The key is `name`.
+ The duplicate-name rule appears three times (`config.typ:39`, `:216`, `rule-packs.typ:93`), and `config.typ:820-827` is a changelog paragraph.


== TO-BE

- 🟢 /config/ covers config files, layers and the generated section map.
- 🟢 /statuses/ covers the legend: values, transitions, notes, `unblocked`.
- 🟢 /rules/ covers `[[tasks.rules]]`, body rules, `check` functions and WIP caps.
- 🟢 Link kinds move to /links/. `[[ls.rules]]` has one home and the other pages link to it.
- 🟢 Every TOML block is a slice of a checked fixture, and the pages follow the plain-prose rules.


== Source of truth

- `schema/config.schema.json` and `crates/mindtape/src/config*.rs`: keys and defaults.
- `mt config show --origin` on a fixture: what each layer contributes.


== Changes

- `www/content/config.typ`: cut to files, layers and the section map.
- `www/content/statuses.typ`, `www/content/rules.typ`, new.
- `www/content/links.typ`, `www/content/cli.typ`, `www/content/filters.typ`: take or drop the moved sections.
- `www/utils/site.typ`: NAV entries.
- `examples/*/.mindtape/config.toml`: `docs:slice` markers for every TOML block the pages show.
- `.just/scripts/check-doc-blocks.sh`: drop `config.typ` from the list.


== Out of scope

- `www/content/rule-packs.typ`: it keeps its own page; the reference-page task rewrites it.


== Dogfood

Config changes in this repo are checked against these pages, since the slices come from configs `just check-examples` validates.

