#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Add nested field properties to filter and sort",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  estimate: (files: 14),
  assignee: "agent",
  tags: ("cli",),
  status: done(
    2026,
    9,
    15,
  )[status.name/date/note/index now filter and sort; --sort derives from the registry],
  links: related(
    "filter-and-sort-parity-for-all-standard-columns.typ",
  )[the registry this extends],
)

== Summary

`status` is one field carrying four values — the lifecycle name, the stamp
date, the stamp note, and the legend position — but only the name is
comparable and only the position is sortable today, so a task's stamp date is
reachable only through the `stale(N)` predicate. Give every fixed named part
of a field its own address, `field.property`, and let both filter and `--sort`
read it through the one registry that already backs every other field.

== Scope

- Add `status.name`, `status.date`, `status.note`, and `status.index` to
  `crates/mindtape/src/fields.rs`. Bare `status` keeps the split it has today:
  a comparison reads the lifecycle name, an ordering reads the legend
  position, so `status == "wip"` keeps working unchanged.
- Derive `--sort`'s accepted vocabulary from the registry plus `ready`
  instead of the parallel `SORT_FIELDS` const, so a field or property added to
  the registry is sortable by construction and no second list has to
  remember it.
- Tests for the four paths in `fields_tests.rs`, `filter_tests.rs`, and
  `sort_tests.rs` / `sort_compare_tests.rs`: ISO date order, note text,
  numeric index, a property `Missing` when unset, and the filter helpers
  (`has` / `miss` / `contains` / `matches`) reaching a property.
- Docs: `www/content/filters.typ`, `docs/src/skill.typ` (regenerated
  `skills/mindtape/SKILL.md`), and `CHANGELOG.md`; re-record the `cli-docs`
  outdatty group.
- No task field changes shape.

== Dogfood

The backlog can sort by the stamp itself (`mt ls --sort status.date:desc`) and
query its parts (`status.date < now()`, `has(status.note)`) instead of
approximating recency with `stale(N)` buckets. Every existing
`.mindtape/config.toml` filter that compares `status` keeps matching, because
bare `status` still reads the lifecycle name.

== Decisions

- Property names mirror the serialized shape (`status.name`, `status.date`,
  `status.note`) plus one derived part (`status.index`, the legend position).
  Bare `status` keeps the split it has always had — a comparison reads the
  name, an ordering reads the position — rather than becoming an alias of one
  property: `status == "wip"` is the shipped in-flight default and the
  spelling throughout the docs and presets, and comparing a numeric alias
  against a non-numeric literal falls back to string-comparing the number in
  `filter::compare`, so an alias would have matched nothing, silently.
- `--sort`'s vocabulary is derived from `crate::fields` (`is_sort_field`
  accepts every registry name plus `ready`) instead of the parallel
  `SORT_FIELDS` list and the test that policed it. `role` stays filter-only:
  it has no value a comparator could order.
- `status.index` is `Missing` for an unset status or a name the legend does
  not carry, not the `usize::MAX` sentinel `sort::status_index` uses: a filter
  must never compare against a sentinel number, and a missing value already
  sorts last.
- No task field changes shape; collections (`links`, `estimate`, `tags`) stay
  out of the property mechanism until an aggregate meaning exists for them.

== Outcome

- `crates/mindtape/src/fields.rs` carries the four `status.*` entries;
  `crates/mindtape/src/sort.rs` drops `SORT_FIELDS` for `is_sort_field`.
- Tests: `fields_tests.rs` (values, missing, out-of-legend index, bare-name
  split), `filter_tests.rs` (comparisons plus `has`/`miss`/`contains`/
  `matches`), `sort_compare_tests.rs` (`status.name` alphabetical,
  `status.index` legend order, `status.date` ISO order and missing-last).
- Docs: `www/content/filters.typ` and `docs/src/skill.typ` (regenerated
  `skills/mindtape/SKILL.md`), `CHANGELOG.md`; the `cli-docs`, `cli-name`,
  `prelude-docs`, and `release-notes` outdatty groups re-recorded after the
  final edit.
- A review round after the first commit caught a stale `outdatty.lock` hash
  (the skill line was compacted after the first re-record) and a docs
  over-claim that every property takes `contains`/`matches`; both corrected,
  with `status.date:desc` and `status.note` sort tests added.
- `just check` passes. `check-file-size` caught `docs/src/skill.typ` at
  324/320 from the first draft of the skill sentence; it was compacted to the
  cap, not the cap raised.
- Estimate revised from 5 files to the 14 paths the change touched (tests,
  docs, and the lock included). The decision stayed in one module; the rest
  was mechanical fan-out.
