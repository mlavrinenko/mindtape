#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "render the assignee as an ls column",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  assignee: "agent",
  tags: ("cli", "www"),
  estimate: (files: 6),
  links: related("adopt-the-assignee-field-in-the-dogfood-backlog.typ")[
    the adoption that made the missing column visible
  ],
  status: done(
    2026,
    8,
    18,
  )[table and CSV both render it, between `due` and `title`; `auto` visibility
    resolves on whether the rows disagree, so a listing already narrowed to one
    actor grows no column],
)

== Summary

`assignee` is filterable (`assignee == "max"`, `role == "maintainer"`) and sortable (`--sort assignee`), but it cannot be displayed: `LS_COLUMNS` in `crates/mindtape/src/config_ls.rs` holds seven names — `#`, `status`, `ready`, `priority`, `due`, `title`, `file` — and `--columns '#,title,assignee'` fails with `unknown column`. So a mixed listing can be filtered by who works each task but never shows it, and the only way to read the field off a listing is `--json`.

Filter/sort parity was closed for this field already (see the related parity task); the render plane was not.

== Scope

- `assignee` into `LS_COLUMNS`, `ColumnId` and the label/width/visibility mapping in `crates/mindtape-cli/src/render/columns.rs`
- `auto` visibility, like `file`: a single-assignee listing (`mt ls @agent`) must not grow a column that repeats one name on every row
- the docs that enumerate the column vocabulary, plus whichever `www/generated/` captures the new column reaches
- out of scope: `difficulty`, `estimate`, `tags` and `reviewed`, which share exactly this gap. One column at a time says whether `auto` visibility is the right rule for a short scalar before four of them land on it.
