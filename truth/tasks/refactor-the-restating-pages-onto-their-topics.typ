#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refactor the restating pages onto their topics",
  kind: "docs",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (files: 9),
  assignee: "agent",
  status: wip(
    2026,
    9,
    18,
  )[NAV groups by reader job; config, cli, dsl, concepts, index and named-sets each say their own part once],
  x-goals: (
    product: (
      readable-tasks: helps[a reader lands on one page per fact instead of four that disagree by inches],
      implementable-tasks: neutral,
      enforced-practice: helps[the audit pairs are closed by making a page delegate, and the same recipe proves each rewrite],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[six cross-file duplicate pairs get one home each; a fact moved to its topic cannot keep a second copy behind],
    ),
  ),
)

== Problem

1. `www/content/config.typ` is 874 lines over 21 headings. The deep dives
  that used to live inside it (hooks, estimates, views and dictionaries, rule
  packs, priority, filters) are pages of their own now, and the page still
  explains topics those pages own.
2. `www/content/cli.typ`'s narrative sections repeat `www/utils/cli-notes.typ`
  word for word in places: `just dup-prose` reports three pairs over half
  overlap, plus one with `/links/`'s subject and one with `/named-sets/`.
3. 🔴 The site's NAV groups pages by where they came from (`Guide`,
  `Reference`, `More`) rather than by the reader's job, so `/comparison/` sits
  beside `/internals/` and `/links/` sits under `Guide`.

== TO-BE

- 🟢 `NAV` groups by reader job: Start, Model, Tasks, Reference, Integrate.
- 🟢 `config.typ` keeps the map and the config-only surface; every topic with
  a page gets one pointer line.
- 🟢 `cli.typ` keeps the generated tree and transcripts; its prose frames
  relations and links out per-command detail.
- 🟢 `/dsl/` keeps fields, status and bodies; links and recurrence are
  pointers, and the enforcement split lands on `/concepts/`.
- 🟢 The opening demo moves to `www/utils/demo.typ`, rendered by `/` now and
  imported by README in the markdown wave.
- 🟢 `just dup-prose` after names no pair whose home is one of these pages.

== Source of truth

- `docs/src/contributing/documentation.typ` — refactor whole, one home per
  fact, checked example beats description.
- `just dup-prose` — the pair list this task closes, and the before/after.
- The topic pages themselves — each already carries the fact `config.typ` and
  `cli.typ` restate.
- `www/utils/site.typ` — the NAV comment on why the grouping is a judgement
  no page owns.

== Changes

- `www/utils/site.typ` — the five reader-job groups.
- `www/content/config.typ` — full rewrite: layers, sections map, and the
  config-only keys; pointer lines for the rest.
- `www/content/cli.typ` — narrative thinned to framing plus cross-links.
- `www/content/dsl.typ` — the delegation, and the model sections moved out.
- `www/content/concepts.typ` — the model home, including the enforcement split
  that used to close the DSL page.
- `www/content/index.typ`, `www/utils/demo.typ` (new) — the pitch and the
  shared opening demo.

== Out of scope

- README.md's side of the shared demo. `www/utils/demo.typ` exists so the
  markdown wave can switch `docs/src/readme.typ` to it without touching the
  site page again.
- The remaining Reference and Integrate pages (schema, agents, internals,
  templates, estimates, hooks, priority, filters, rule-packs, import-export,
  quickstart, comparison, derived-tags, self-validating). The next wave owns
  them.

== Dogfood

`just dup-prose` before and after; the rewrite is whole-file, so no section
survives because deleting it was inconvenient.
