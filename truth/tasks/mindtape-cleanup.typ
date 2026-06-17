#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mindtape cleanup: retire terminal tasks, rewrite links to SWHID",
  status: done(2026, 6, 17),
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

- Refuse on a dirty working tree (real run only); resolve tombstones before
  deleting (the working blob must still exist to verify recoverability).
- Atomic: plan every file edit and mint every tombstone up front; if any affected
  file is wrong-shaped or any commit cannot be resolved, abort with no writes.
- No commit: deletions and link rewrites are applied in place; the user reviews
  the diff and commits. `--dry-run` previews the plan and writes nothing (and
  skips the dirty-tree guard).
- Link model change: `swh:` targets are history pointers — the loader
  (`crates/mindtape/src/model.rs` `parse_links` via `is_history_pointer`) drops
  them from the normalized edge lists, so `validate` never resolves them.

== Acceptance

`mindtape cleanup` on the dogfood backlog removes terminal tasks and rewrites
inbound links to valid SWHID tombstones; `mindtape check` accepts the rewritten
links; `just check` passes; tests cover the dirty-tree refusal and the
all-or-nothing abort.
