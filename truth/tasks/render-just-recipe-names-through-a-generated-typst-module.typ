#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "render just recipe names through a generated typst module",
  priority: ice(
    impact: 9,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("docs", "tooling", "gating"),
  estimate: (files: 4),
  links: depends-on(
    "give-every-just-recipe-a-doc-attribute.typ",
    "tag-just-recipes-with-groups.typ",
  )
    + related(
      "convert-hand-typed-just-mentions-to-the-generated-module.typ",
    )[the sibling that rewrites existing prose onto this mechanism],
  status: proposed(2026, 8, 14),
)

== Summary

Generate a Typst module from `just --dump --dump-format json` whose members are the recipes, so a doc source writes `#just.check()` and `#just.mutants("-f <file>")` instead of hand-typing the name. Build the mechanism here; the conversion of existing prose is a sibling task.

== Why

29 distinct `just <recipe>` mentions live across `docs/src`, `www/content`, `README.md` and `SKILL.md` — `just check` alone appears 15 times — and nothing verifies a single one resolves to a real recipe. Rename a recipe and every mention rots silently. That is the same failure class the derived-facts work was built to kill, still wide open, and it is about to matter more: the module split renames recipes wholesale.

A generated module makes the failure loud for free. Verified in the dev shell: a missing member errors with "module just does not contain nope" at render time, naming it. No new gate is needed for the inline case — `just docs-md` fails on its own.

Field access must go through a *module*, not a dict: Typst reads a dict field call as a method call and errors "type dictionary has no method". A generated `.typ` file imported with an `as just` alias calls cleanly, hyphenated names included (verified with a name carrying two hyphens).

== Scope

- `www/generate-facts.sh` emits `www/generated/just.typ`, one `#let` per public recipe binding a function that renders the invocation as a raw span. Extra positional args join onto the command, so `#just.mutants("-f <file>")` renders "just mutants -f \<file\>".
- Identifier mangling is the *invocation path* joined with `-`, so the module split can move `docs-md` to module `docs` recipe `md` without touching one line of prose. Two recipes mangling to the same identifier fails the generator.
- `docs/src/lib.typ` re-exports it so a source imports one place; `www/content/*.typ` can import it directly. Both render after `www/generate-facts.sh`, so the file always exists — but `www/generate.sh` wipes `www/generated/` on entry, so ordering has to be confirmed, not assumed.
- A fenced code block cannot call the helper. New `check-doc-recipes` gate arm instead: fail on any literal "just \<name\>" in a docs source whose name is not a real recipe, and on any literal one *outside* a fenced block (that one must use the helper).
- Bare `just` with no recipe ("`just` with no args lists them") stays literal — the ban is on a name following it.
