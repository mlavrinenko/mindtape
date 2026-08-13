#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "derive the gate table command column from the justfile",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("docs", "tooling"),
  estimate: (files: 5),
  links: depends-on(
    "render-just-recipe-names-through-a-generated-typst-module.typ",
    "tag-just-recipes-with-groups.typ",
  ),
  status: proposed(2026, 8, 14),
)

== Summary

Split CONTRIBUTING.md's Quality Gates table into a derived column and an editorial one: the command each gate runs comes from the Justfile, the rationale moves to a `www/utils/gate-notes.typ` prose dict like every other notes dict in this repo.

== Why

The Tool column hand-quotes commands that the JSON dump already carries in `.recipes[].body`, and one has already drifted:

- `docs/src/contributing.typ` says `cargo clippy --all-targets -D warnings`
- Justfile runs `cargo clippy --workspace --all-targets -q -- -D warnings`

Missing `--workspace`, missing `-q`, missing the `--` separator. Nobody caught it — the same way nobody caught "three substrate crates" when there were seven.

Five rows (`clippy`, `test`, `machete`, `check-file-size`, `outdatty-check`) are 100% duplication of a one-line body. The rest carry real argument that has no machine source and must stay prose — but inline in the doc source, which is the odd one out: `crate-notes.typ`, `dsl-field-notes.typ` and `cli-notes.typ` all live under `www/utils/` and are all set-differenced by a gate.

== Scope

- Command column: `.recipes[].body` when the recipe is non-shebang and short, otherwise the `[doc(...)]` string. `just --dump --dump-format json` already runs in `www/generate-facts.sh`; extend `gates.json` rather than adding a second dump.
- Move the `gate-tools` dict out of `docs/src/contributing.typ` into `www/utils/gate-notes.typ`, one entry per gate, rationale only — no restated command.
- Third arm on `just check-doc-facts`: every gate in `gates.json` needs a `gate-notes.typ` entry, same set-difference idiom as the crate and DSL-field arms.
- Cosmetic follow-up this closes: the Gate column no longer duplicates the recipe name that opens its Tool column.
