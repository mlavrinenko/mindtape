#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Render the filter field table from the field registry",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("docs", "www"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent writing a filter reads the fields and which of them sort off the code that resolves them],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[a field added, renamed or made sortable changes the page in the same build],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 17),
  links: parent("keep-the-user-docs-generated-or-gated-not-restated.typ")
    + depends-on(
      "list-the-template-slot-kinds-from-one-slotkind-registry.typ",
    )[both add a registry to tree\_json.rs and generate-facts.sh, so landing in turn avoids a conflict in the same lines]
    + depends-on(
      "split-the-cli-docs-drift-group-by-the-prose-each-source-feeds.typ",
    )[it edits the outdatty group the split creates for fields.rs],
)

== Problem

+ Open the field table in `www/content/filters.typ` and the field list in `skills/mindtape/references/filters.md`.
+ 🔴 Both are typed by hand, and `filters.typ` says "Every field in the table sorts except `role`, `estimate.<unit>` and `estimate`" in prose. `check-doc-facts` checks the filter functions' names on both pages, but no arm reads `FIELDS` in `crates/mindtape/src/model/fields.rs`, so a renamed field leaves both green.


== TO-BE

- 🟢 `mt _cli --json` carries `fields`: every `FIELDS` entry, each marked sortable by `mindtape::sort::is_sort_field`.
- 🟢 The filters page renders its field table from that fact and `www/utils/filter-field-notes.typ`. A note whose name is not in `FIELDS`, such as `role` or `tags`, renders as filter only, so the sort sentence comes from the data.
- 🟢 The build fails on a field with no note, and `check-doc-facts` fails when `filters.md` names no span for a field.


== Source of truth

- `crates/mindtape/src/model/fields.rs`: `FIELDS`, the comparable and sortable fields.
- `crates/mindtape/src/sort.rs`: `is_sort_field`.


== Changes

- `crates/mindtape-cli/src/cli/tree_json_tests.rs`: the red test first: `fields` holds every `FIELDS` name with `sortable: true`.
- `crates/mindtape-cli/src/cli/tree_json.rs`: export `fields`, as `filter_functions` does.
- `www/generate-facts.sh`: write `filter-fields.json` from `cli.json`.
- `.just/scripts/check-doc-facts.sh`: an arm like the filter-functions one, over `filters.md`.
- `www/utils/filter-field-notes.typ`: one note per field, the filter-only ones included.
- `www/content/filters.typ`: render the table and the sort sentence from the fact and the notes.
- `outdatty.yaml`: drop `model/fields.rs` from the group whose prose this table replaces, keeping `filters.md`'s pairing.


== Out of scope

- The function table's meanings and arities, which keep the name check they have.
- Filter-only fields as a registry of their own. They live in the notes until a second consumer needs them.


== Dogfood

This repo's `just check` builds the filters page, so a field change in `fields.rs` fails here until it has a note.

