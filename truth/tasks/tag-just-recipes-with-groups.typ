#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "tag just recipes with groups",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling", "docs"),
  estimate: (files: 2),
  links: depends-on("give-every-just-recipe-a-doc-attribute.typ"),
  status: proposed(2026, 8, 14),
)

== Summary

Use just's `[group(...)]` attribute as a tagging system over recipes, and make `[group("gate")]` plus Justfile source order the source of the gate table — instead of unwrapping the `mmz` indirection out of `check`'s dependency list.

== Why

A recipe can carry several `[group(...)]` attributes and appears under each. Verified on just 1.43.1: the dump exposes them as `attributes: [{"group": "gate"}, ...]`, `just --groups` lists the names, `just --list` renders them as headings. That is a tagging system with no new machinery.

Source order is available too, and is not in the JSON dump (which sorts recipes alphabetically): `just --summary --unsorted` returns names in source order. Ordering the gate recipes in the Justfile the way the table should read makes the Justfile the single source of both membership and order.

== Scope

- Groups to introduce: `gate` (the arms of `just check`), `agent` (what an agent may run unsupervised), `docs`, `advisory` (the not-a-gate signals: `ask-refactoring`, `mutants*`, `cover`, `crap`, `file-sizes`), `release`.
- Reorder the gate recipes in the Justfile so source order is the order the CONTRIBUTING.md table should read.
- `www/generate-facts.sh`: build `gates.json` from group membership intersected with `just --summary --unsorted` order, not from `check`'s dependency list.
- Keep a cross-check, because the two can disagree: a recipe tagged `gate` but absent from `check`'s dependencies (or the reverse) fails the generator by name. That is the assertion the old derivation got for free and must not lose.
- Groups are advisory metadata for docs and discovery; nothing about `just check`'s parallel execution changes.
