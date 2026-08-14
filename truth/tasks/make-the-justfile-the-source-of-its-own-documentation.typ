#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "make the justfile the source of its own documentation",
  priority: ice(
    impact: 9,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 6,
  kind: "umbrella",
  tags: ("docs", "tooling"),
  estimate: (files: 15),
  links: related(
    "give-every-just-recipe-a-doc-attribute.typ",
    "tag-just-recipes-with-groups.typ",
    "gate-the-justfile-format-with-just-fmt.typ",
    "adopt-just-attributes-for-cwd-privacy-and-confirmation.typ",
    "render-just-recipe-names-through-a-generated-typst-module.typ",
    "convert-hand-typed-just-mentions-to-the-generated-module.typ",
    "derive-the-gate-table-command-column-from-the-justfile.typ",
    "derive-the-formatter-path-lists-from-justfile-assignments.typ",
    "split-the-justfile-into-modules.typ",
  )[the nine leaves of this milestone],
  status: done(
    2026,
    8,
    14,
  )[all nine leaves landed, plus the www/generated race the milestone exposed. just --list is a grouped menu, every gate row's command is read from its recipe body, and no doc source hand-types a recipe name — check-doc-recipes fails on one],
)

== Summary

The derived-facts pipeline reads `just --dump --dump-format json` already, and throws half of it away. Close that: make every fact the root docs state about a recipe — its description, its membership in the gate set, the command it runs, the name it is called by — read from the Justfile instead of being restated beside it.

== Why this is one milestone and not nine tasks

The leaves share one root cause. just takes the *last comment line* before a recipe as its doc, this Justfile argues its recipes in long comment blocks, and so the doc field is the tail fragment of a paragraph — 22 of 45 recipes advertise one in `just --list`. With no usable machine description, `docs/src/contributing.typ` hand-wrote its whole gate table, and the hand-written column has already drifted from the recipe it describes (its `clippy` row is missing `--workspace`, `-q`, and the argument separator).

Fix the doc attribute and the rest unlocks in order: a trustworthy description, then groups as a tagging system, then the command column, then recipe names in prose, then the module split that renames everything and would otherwise rot all 29 hand-typed mentions at once.

== Sequencing

The Justfile and `www/generate-facts.sh` are shared hubs — every leaf touches at least one — so these run as sequential waves, not parallel worktrees.

+ Justfile hygiene: format gate, doc attributes, groups plus the group-derived gate list, the cwd/private/confirm attributes.
+ The generated Typst module of recipe names, then the conversion of existing prose onto it.
+ The gate table's derived command column and the formatter path lists.
+ The module split, last, because it renames recipes and only the generated module makes that safe.

== Done when

`just --list` reads as a coherent menu, `www/generated/` carries every fact the root docs state about a recipe, and no doc source hand-types a recipe name.
