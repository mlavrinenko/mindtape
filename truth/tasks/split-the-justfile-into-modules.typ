#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "split the justfile into modules",
  priority: ice(
    impact: 6,
    confidence: 0.6,
    ease: 3,
  ),
  difficulty: 7,
  tags: ("tooling",),
  estimate: (files: 10),
  links: depends-on(
    "render-just-recipe-names-through-a-generated-typst-module.typ",
    "derive-the-gate-table-command-column-from-the-justfile.typ",
  ),
  status: done(
    2026,
    8,
    14,
  )[one module, docs. Every other cluster failed the read-better test and stayed at top level. The identifier mangling held: docs::md mangles to docs-md, so no doc SOURCE changed — two rendered lines did, where the invocation text itself moved from 'just docs-md' to 'just docs md'. Row order moved off 'just --summary --unsorted' (which lists every top-level recipe before any module recipe, so the two docs gates would have jumped to the end of the table) onto check's own dependency list],
)

== Summary

Break the 726-line Justfile into just modules — `docs`, and the other cohesive clusters — so a recipe is found by area instead of by scrolling.

== Why, and the honest cost

Verified on just 1.43.1: a module recipe *can* be a dependency (`top-check: docs::build` runs), invocation works both as `just docs build` and `just docs::build`, and the dump nests them under `.modules`.

The cost is that a rename touches every reference, and the references are not all in docs:

- `.mmz/config.yaml` keys every memoized rule by its `just <recipe>` name
- `check`'s arms are `(mmz "<name>")`, so each becomes `(mmz "docs::check")`
- `.github/workflows/{ci,pages,release}.yml` — five call sites
- `contrib/ask-refactoring/{coverage,mutants}.sh` — four mentions
- `flake.nix` shell hook calls `just install-hooks`

Docs prose is the one surface this does *not* touch, and only because the generated-module task mangles an invocation path to a single identifier: `docs::md` mangles to `docs-md`, which is what prose already calls it. That is why this task depends on that one.

== Scope

- Do not modularize `check` — the aggregate gate keeps its bare name, and a module named `check` would collide with it.
- Candidate modules: `docs` (gen, build, serve, check, md, md-check), and whichever further clusters survive the same test — does the module name plus recipe name read better than the hyphenated name it replaces? If it does not, leave the recipe at top level. A split that produces `just cov run` where `just cover` stood is not an improvement.
- Update every reference listed above in the same commit; a half-renamed reference fails at CI time, not at gate time.
- Regenerate `www/generated/just.typ` and confirm the mangled identifiers are unchanged for every recipe that moved — a prose diff after this task means the mangling is wrong.
