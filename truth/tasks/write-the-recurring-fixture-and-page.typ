#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write the recurring fixture and page",
  kind: "docs",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (files: 15),
  assignee: "agent",
  status: done(
    2026,
    9,
    18,
  )[examples/recurring is checked by just check-rrule, the page reads its slices through the shared www/utils/slices.typ, and /dsl/ delegates; the feature-build race that moved the arm out of check-examples is argued in the Justfile],
  x-goals: (
    product: (
      readable-tasks: helps[the recurrence reference is real tasks read back as data, including the pair a spawn leaves behind],
      implementable-tasks: neutral,
      enforced-practice: helps[the rrule arm runs with the cargo feature enabled, so the escape hatch is proven rather than described],
      cheap-adoption: helps[a small ops-shaped example shows the whole recurrence surface in one page],
      typst-over-markdown: helps[every() and rrule() are shown as they are authored, with the configured policy floor beside them],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the page's blocks are slices of a checked fixture, and the old DSL section cannot keep its own copy],
    ),
  ),
)

== Problem

1. `www/content/dsl.typ`'s "Recurring tasks" section is a syntax list with
  invented task shape around it; nothing checks that the list still matches
  what a task file may declare.
2. `rrule(...)` needs the cargo feature and `[tasks.recur] rrule` together, and
  no checked fixture carries either, so the documented rejection path lives
  only in prose.
3. 🔴 The pair `mt recur` leaves behind (a successor with `recurred-from`) is
  not in any example; `recurred-from` appears on `/links/` with a bare link
  and no real file to show.

== TO-BE

- 🟢 `examples/recurring/` is a checked ops-shaped project: one task per period
  unit, one carrying the full policy (`anchor`, `until`, `count`,
  `materialize`), one `rrule(...)` task, and a spawned pair with the
  `recurred-from` edge.
- 🟢 Its `mt check` runs in `just check-rrule` beside the feature build,
  never in check-examples: the two cargo feature sets write the same
  `target/debug/mt`, and the default-feature test arm runs in parallel.
- 🟢 `www/generate-examples.sh` slices it into `www/generated/recurring.json`,
  and `www/content/recurring.typ` renders those slices.
- 🟢 `www/content/dsl.typ`'s recurring section becomes a pointer to the page,
  and `/links/` points at it for `recurred-from`.
- 🟢 `just dup-prose` reports no pair whose home is the new page.

== Source of truth

- `lib/prelude.typ` — `every()`, `rrule()`, the weekday constants, and `on()`
  for dates.
- `www/content/config.typ` — the `[tasks.recur]` floor and the two gates the
  escape hatch needs.
- `www/generate-examples.sh` — the slicing and marker convention this reuses.
- `examples/links/` — the fixture shape this one follows, including the
  `[[typst.local-packages]]` override.

== Changes

- `examples/recurring/` (new) — config with `rrule = true` and seven tasks.
- `www/generate-examples.sh` — slice both fixture directories, one fact each.
- `Justfile` and `.mmz/config.yaml` — the check-rrule arm (with the race it
  avoids argued beside it) and the typst-fmt scope.
- `www/content/recurring.typ` (new), `www/utils/site.typ` — the page and its
  nav entry.
- `www/content/dsl.typ` and `www/content/links.typ` — the delegating edits the
  new page pays for.

== Out of scope

- The page regroups and the rest of the `/dsl/` rewrite; wave 3 owns them.
- Running `mt recur` inside the generator. The fixture authors the pair the
  command writes, and the CLI behavior stays on `/cli/`.

== Dogfood

The page's blocks are derived slices, `just dup-prose` runs before and after,
and the old section it replaces is deleted rather than left beside it.
