#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Reject an unknown sort direction",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 2,
  estimate: (files: 4),
  assignee: "agent",
  tags: ("cli",),
  status: wip(2026, 9, 15),
)

== Summary

`mt ls --sort priority:wtf` sorts by priority ascending and says nothing.
`validate_sort_spec` splits the spec on the first `:` and validates only the
field, and `parse_spec` maps every suffix other than `desc` to ascending, so a
misspelled direction is indistinguishable from `asc` until someone reads the
order and notices. Hit live as `--sort status.date:wtf`.

== Scope

- `crates/mindtape/src/sort.rs`: validate the suffix in `validate_sort_spec`
  (`asc`, `desc`, or no colon at all) and have `sort_by_specs` delegate to it
  instead of re-checking the field alone; a bad direction errors naming the
  direction and the two accepted spellings.
- Tests in `sort_tests.rs`: `validate_sort_spec` rejects `priority:wtf` and
  `priority:` and accepts `priority`, `priority:asc`, `priority:desc`;
  `sort_by_specs` surfaces the same error.
- Document it in `www/content/filters.typ` and `CHANGELOG.md`.
- `[ls] default-sort` takes the same validator, so a saved typo fails at
  config load too.

== Dogfood

Every `--sort` in this repo's views and scripts still validates because each
spells `asc`/`desc`; the next typo is a loud error rather than a listing that
quietly ignores the direction.
