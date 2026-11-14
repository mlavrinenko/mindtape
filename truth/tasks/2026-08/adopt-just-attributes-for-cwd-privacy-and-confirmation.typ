#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "adopt just attributes for cwd privacy and confirmation",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 1),
  status: done(2026, 8, 14),
)

== Summary

Take the three remaining recipe attributes that replace hand-rolled equivalents:
`[working-directory]` for the `cd www &&` prefix, `[private]` for the
underscore-prefixed recipes, and `[confirm]` for `release`.

== Why

`docs-build`, `docs-serve` and `docs-check` all open with `cd www && ...`, which
turns a clean command list into one shell string. A one-line body is exactly
what the gate table needs to quote. `[working-directory("www")]` works on just
1.43.1.

== Scope

- `[working-directory("www")]` on the three `docs-*` recipes that `cd www`
  today. Drop the prefix from each body.
- `[private]` on `_crap-excludes` and `_lcov-stale-reason`. The underscore then
  stops mattering, though renaming them is out of scope here.
- `[confirm("...")]` on `release`, which today relies only on in-recipe
  assertions.
- Anything that shells out to these recipes
  (`contrib/ask-refactoring/coverage.sh` calls `just _lcov-stale-reason`
  indirectly via `check-crap`) must keep working. `[private]` hides a recipe
  from the list without blocking a call. Verify rather than assume.
