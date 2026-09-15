#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add a docs page on tasks that validate themselves",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (files: 7),
  kind: "docs",
  assignee: "agent",
  status: proposed(2026, 9, 15),
)

== Problem

+ Run `rg -n 'import "\.\./[^/"]+\.typ": \*' www/content examples`.
+ 🔴 It prints nothing. No page or example shows a task importing a shared Typst file, the one thing Markdown with frontmatter cannot do without writing a linter.


== Goals

#goals(
  product: (
    readable-tasks: neutral,
    implementable-tasks: neutral,
    enforced-practice: helps[readers see a rule enforced by the task files themselves, with no linter to maintain],
    cheap-adoption: neutral,
    typst-over-markdown: helps[the page proves the claim with a live refusal instead of prose],
    plain-files: neutral,
  ),
  process: (
    sound-codebase: neutral,
    docs-match-code: helps[the page quotes a live capture, never a hand-typed transcript],
  ),
)

== TO-BE

- 🟢 A docs page shows a shared goals file, a task importing it, and the live `mt check` output refusing a task that skips a goal.
- 🟢 The page sits in the site nav, and `www/generate.sh` regenerates its capture.

== Source of truth

- The fixture files and the `mt check` capture `www/generate.sh` writes: the page quotes both, never retypes them.
- `truth/lib.typ` and `truth/goals.typ`: the fixture follows their split of code from data.


== Changes

- `examples/self-validating/` (new): a minimal config, a `goals.typ` and a `lib.typ`, one task that passes and one that skips a goal.
- `www/generate.sh`: capture `mt check` against that fixture beside the learn-piano captures.
- `www/content/self-validating-tasks.typ` (new): the page, quoting the fixture files and the capture.
- `www/utils/site.typ`: register the page in the nav.


== Out of scope

- A failing task in `examples/learn-piano`: `just check-examples` must stay green there.
- The README: `rewrite-the-readme-around-what-typst-tasks-express.typ`.


== Dogfood

Built in: the fixture's `goals.typ` follows this repo's `truth/goals.typ`.

