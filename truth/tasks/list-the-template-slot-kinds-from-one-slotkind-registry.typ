#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "List the template slot kinds from one SlotKind registry",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("docs", "www", "cli"),
  estimate: (src-files: 6),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a template author who misspells a kind reads every kind `mt add` accepts],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the templates page and both refusals read the kinds off the enum that parses them],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17),
  links: parent("keep-the-user-docs-generated-or-gated-not-restated.typ"),
)

== Problem

+ Put `tags: slot("tags", kind: "multi-choice", choices: ("a", "b"))` in `.mindtape/templates/task.typ` and run `mt add x`.
+ 🔴 It prints `error: slot declares an unknown kind: 'multi-choice' in slot 'tags': expected one of string, int, float, real, bool, date, list, text, raw, choice`. `multichoice` is missing, though `SlotKind::parse` accepts it.
+ 🔴 The list is typed out in `crates/mindtape/src/template/fill/mod.rs:141` and `crates/mindtape-cli/src/diag/from_lib.rs:438`, and again in the `kind` row of `www/content/templates.typ`. No gate or outdatty group ties any of them to `crates/mindtape/src/template/mod.rs`.


== TO-BE

- 🟢 `SlotKind::ALL` lists every kind once, `parse` and both refusals read it, and the refusal names `multichoice`.
- 🟢 `mt _cli --json` carries `slot_kinds`, and the templates page renders the `kind` row from it, with a note per kind from `www/utils/slot-kind-notes.typ`.
- 🟢 The docs build fails on a kind with no note, the way `config-gate` fails on a section the table leaves out.


== Source of truth

- `crates/mindtape/src/template/mod.rs`: `SlotKind` and the spelling of each kind.
- `mt _cli --json`, which hands the registry to the docs build.


== Changes

- `crates/mindtape/src/template/tests_fill_errors.rs`: the red test first: the `UnknownKind` message names every kind in `SlotKind::ALL`.
- `crates/mindtape-cli/src/diag/tests_fill.rs`: the same for the rendered refusal.
- `crates/mindtape-cli/src/cli/tree_json_tests.rs`: `slot_kinds` holds every kind.
- `crates/mindtape/src/template/mod.rs`: add `SlotKind::ALL` and parse through it.
- `crates/mindtape/src/template/fill/mod.rs`, `crates/mindtape-cli/src/diag/from_lib.rs`: build the list from `SlotKind::ALL`.
- `crates/mindtape-cli/src/cli/tree_json.rs`: export `slot_kinds`, as `date_tokens` does.
- `www/generate-facts.sh`: write `slot-kinds.json` from `cli.json`.
- `www/utils/slot-kind-notes.typ`: one note per kind.
- `www/content/templates.typ`: render the `kind` row from the fact and the notes.


== Out of scope

- The other slot arguments in that table, which `lib/prelude.typ` declares and `prelude-docs` tracks.


== Dogfood

`mt add -t docs --describe` here lists `tags` as a `multichoice` slot, so this repo files tasks through the kind the refusal leaves out, and its docs build fails on a new kind until it gets a note.

