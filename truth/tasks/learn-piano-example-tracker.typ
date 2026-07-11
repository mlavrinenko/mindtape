#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Learn-piano example tracker",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  tags: ("docs",),
  status: proposed(2026, 7, 11),
)

== Summary

A real, self-contained MindTape project under `examples/learn-piano/` — a piano
practice tracker that showcases MindTape features and doubles as the fixture the
docs generator (generate-docs-cli-reference) runs `mt` against. One project
serves every injected-output doc surface; it must pass `mt check` and be gated in
CI so it can never rot silently. It imports the real `@local/mindtape:0.2.0`
(enlisted in the outdatty `typst-pkg-version` group).

== Layout

```
examples/learn-piano/
  .mindtape/config.toml
  .mindtape/templates/<default>.typ   # slotted; ICE leaves as slots
  <root>/technique/…                  # scales, arpeggios, sight-reading
  <root>/repertoire/…                 # pieces
  <root>/theory/…                     # circle of fifths, intervals
```

== Config

- Status legend with `to` transition edges — showcases `mt flip` edge rejection.
- Priority: `framework = "ice"`, `allow-manual = false`, labelled bands
  (green/amber/red). ICE reads naturally here: impact = how much it advances your
  playing, confidence = how reliable the method is, ease = how little effort to
  drill.
- Views: `@stuck` (`blocked && stale(N)`) and `@this-week` — each with a sort and
  a column set, so `mt views` and `mt ls @view` have real content.
- `derive-tags` on the root; the `technique` / `repertoire` / `theory` folders
  produce derived tags (segments + breadcrumbs).
- A default template whose `priority: ice(impact: slot("impact", kind: "int"), …)`
  exposes impact/confidence/ease as slots, plus `title` and `tags` slots — so
  `mt add` fills the nested leaves and `mt add --describe` reads well.

== Content

- Authored tasks spread across the legend (proposed / wip / done / blocked) with
  `due` / `start` dates, tags from the config allow-list, and dependencies that
  form a ready/blocked chain (a repertoire piece `depends-on` a technique task).
- One recurring daily-practice task (`recur: every(days: 1, …)`) so `mt recur`
  has a real subject.
- Enough rows that `mt ls`, a priority-band filter, and the views read as a
  believable backlog.
- Dates authored around the fixed docs clock `2026-07-06` (see pinnable-clock, set
  by the generator) so captured output is stable and reads well.

== Constraints

- Fully-authored literal tasks — no `slot()` survives except in the template file.
- Tags only from the config's `allowed` list.

== Scope

- `config.toml` (legend + transitions, ice + bands, views, derive-tags, estimate
  units).
- the slotted default template (ICE leaves).
- the authored task set (technique / repertoire / theory; deps; a recurring
  task; spread of statuses / dates / tags), dated around `2026-07-06`.
- a `just` recipe running `mt check` on `examples/learn-piano/`, wired into
  `just check`.
- enlist the example's files in the outdatty `typst-pkg-version` group.
