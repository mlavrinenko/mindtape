#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Refactor the audited guide pages onto their topics",
  kind: "docs",
  status: done(
    2026,
    9,
    19,
  )[seven pages refactored. The ls-open captures regenerated as !terminal. The audit sits at the three accepted pairs.],
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (files: 9),
  assignee: "agent",
  x-goals: (
    product: (
      readable-tasks: helps[a page that retells a topic makes the reader choose which copy is true.],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[quickstart keeps the walkthrough and drops the flag catalog.],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the caption matches its capture and the escape list matches status_value.rs.],
    ),
  ),
)

== AS-IS

1. 🔴 An audit of the seven Guide pages the earlier waves never touched found the
  same rot. Quickstart retells `/cli/` and `/config/`. Comparison argues dstask
  three times. `priority` promises a difficulty twin it never delivers. Filters'
  `blocked` definition contradicts `ready.rs`. Derived-tags omits the switch
  that turns it on. Templates' escape list misses two characters. The
  `ls-open` caption ("everything not yet terminal") describes a capture that
  filters `status != "done"`.

== TO-BE

- 🟢 Each guide page states its part once and cross-links the owner. The
  captures in the walkthrough stay.
- 🟢 `www/generate.sh` captures `mt ls '!terminal'` for `ls-open`, so the
  caption is true of the rows below it.
- 🟢 priority covers difficulty. Estimates says the drift point once under its
  own headings. Templates' escape list matches `escape_markup`.

== Source of truth

- `docs/src/contributing/documentation.typ`: one home per fact, whole-page
  refactors.
- `crates/mindtape/src/status_value.rs`, `ready.rs`, `config_roots.rs`,
  `defaults.toml`: the facts the pages must match.

== Changes

- The seven pages, `www/generate.sh`, and the regenerated captures.

== Out of scope

- Turning the hand-typed examples into fixture slices and deriving the
  framework table. A separate task covers it.
- The reference-side pages. The next task in this sweep covers them.

== Dogfood

`tola query` per page while editing, `just dup-prose` before and after, and
`just docs::check` on the regenerated captures.
