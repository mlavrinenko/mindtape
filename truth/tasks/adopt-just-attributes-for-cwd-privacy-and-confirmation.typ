#import "@local/mindtape:0.2.0": *

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

Take the three remaining recipe attributes that replace hand-rolled equivalents: `[working-directory]` for the `cd www &&` prefix, `[private]` for the underscore-prefixed recipes, `[confirm]` for `release`.

== Why

`docs-build`, `docs-serve` and `docs-check` all open with `cd www && ...`, which turns a clean command list into one shell string — and a one-line body is exactly what the gate table wants to be able to quote. `[working-directory("www")]` is verified working on just 1.43.1.

== Scope

- `[working-directory("www")]` on the three `docs-*` recipes that `cd www` today; drop the prefix from each body.
- `[private]` on `_crap-excludes` and `_lcov-stale-reason` — then the underscore is no longer load-bearing, though renaming them is out of scope here.
- `[confirm("...")]` on `release`, which today is guarded only by in-recipe assertions.
- Anything that shells out to these recipes (`contrib/ask-refactoring/coverage.sh` calls `just _lcov-stale-reason` indirectly via `check-crap`) must keep working: `[private]` hides a recipe from the list, it does not make it uncallable — verify rather than assume.
