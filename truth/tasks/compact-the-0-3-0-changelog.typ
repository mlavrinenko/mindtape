#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Compact the 0.3.0 changelog",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("docs",),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a newcomer reads the release notes in minutes],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the release notes say what 0.3.0 ships, once per feature],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 15),
)

== Problem

+ Run `sed -n '/^## \[Unreleased\]/,/^## \[0.2.0\]/p' CHANGELOG.md | wc -c`.
+ 🔴 It prints about 171000. The section logs 0.3.0 commit by commit, so one feature spreads over an Added entry, later Changed entries and a Fixed entry for a bug that never shipped, and many entries name internal modules and gate plumbing.


== TO-BE

- 🟢 `[Unreleased]` is half its size or less, well under the release body's 120 KB budget.
- 🟢 Each feature is one entry describing what ships: a bold headline and up to three short sentences.
- 🟢 Every command, flag, field, config key and predicate a user can reach is still findable.


== Source of truth

- The current `[Unreleased]` entries: every user-facing name stays or folds into its feature's entry.
- `release.toml` and `.github/workflows/release.yml`: the section opens with `### ` and its entries start with `- `, `### ` or `#### `.


== Changes

- `CHANGELOG.md`: rewrite `[Unreleased]` section by section, merging entries per feature and dropping detail no user sees.


== Out of scope

- The released `[0.2.0]` and `[0.1.0]` sections, which stay as shipped.
- The release tooling, which already parses the section.


== Dogfood

The 0.3.0 GitHub release body carries the whole section, uncut.

