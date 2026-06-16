#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mindtape cleanup: retire terminal tasks, rewrite links to SWHID",
  status: proposed(2026, 6, 16),
  priority: "normal",
  estimate-hours: 6,
  depends-on: (
    link("editor-tool-atop-harvest.typ"),
    link("swhid-mint-crate.typ"),
  ),
  tags: ("cli",),
)

== Summary

`mindtape cleanup` removes tasks in a terminal status (the config `terminal` set)
and rewrites every surviving `link("<stem>.typ")` that points at a removed task
into an SWHID tombstone `link("swh:1:rev:<commit>;path=<path>")`, preserving any
`[Label]`. Thin orchestration over `typst-edit` (rewrite) and `swhid-mint` (mint),
mirroring speconaut's proven `cleanup` shape — without the `git-ref` DSL helper.

Note: MindTape's tombstone keeps the link a plain `link(...)` and swaps only the
target string, so `[Label]` survives untouched. No new DSL function is needed.

== Behaviour

- Refuse on a dirty working tree; resolve tombstones before deleting (the working
  blob must still exist to verify recoverability).
- Atomic: plan every file edit and mint every tombstone up front; if any affected
  file is wrong-shaped or any commit cannot be resolved, abort with no writes.
- Single commit: file deletions and link rewrites land together.
- Link model change: `swh:` targets are history pointers — the loader/validator
  (`crates/mindtape/src/model.rs` `normalize_link`/`parse_links`, plus `validate`)
  must skip stem-normalization and existence checks for them and allow a label.

== Acceptance

`mindtape cleanup` on the dogfood backlog removes terminal tasks and rewrites
inbound links to valid SWHID tombstones; `mindtape check` accepts the rewritten
links; `just check` passes; tests cover the dirty-tree refusal and the
all-or-nothing abort.
