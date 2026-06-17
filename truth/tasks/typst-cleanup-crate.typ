#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "typst-cleanup: consumer-blind artifact-retirement orchestration",
  status: done(2026, 6, 17),
  priority: "high",
  estimate-hours: 12,
  relates-to: (
    link("mindtape-cleanup.typ"),
    link("swhid-mint-crate.typ"),
    link("editor-tool-atop-harvest.typ"),
    link("converge-speconaut.typ"),
  ),
  tags: ("substrate", "cli"),
)

== Summary

`mindtape cleanup` (`crates/mindtape-cli/src/cleanup.rs`) and speconaut's
`cleanup_{cmd,refs,tombstones}.rs` run the same pipeline: refuse a dirty tree ->
find eligible (terminal) artifacts -> mint a `swh:1:rev` tombstone per deleted
file (swhid-mint) -> rewrite inbound `link("…")` references in survivors
(typst-edit) -> do not commit; the operator reviews and commits. Same `--dry-run`,
same interactive `confirm`, same all-or-nothing plan-before-write. Both already
ride swhid-mint + typst-edit; the orchestration between them is the duplication.
Host it once as a consumer-blind crate `crates/typst-cleanup`.

== Design — seams, not policy

What is genuinely shared is the orchestration skeleton: dirty-tree guard, mint
tombstone per eligible file, scan survivors for inbound `link()` edges, compute a
non-overlapping `Edit` set, atomic dry-run / confirm / apply, report. What is
NOT shared and must stay consumer-supplied (the user's point — other projects
retire differently):

- Eligibility: which files are terminated. mindtape = terminal task status;
  speconaut = superseded/cancelled decisions and standards plus done/cancelled
  tasks. A trait or closure `eligible(root) -> Vec<EligibleArtifact>`.
- Identity / link resolution: how a `link()` target maps to a file identity
  (here: `Path::file_stem`) — a hook, not baked in.
- Tombstone label synthesis: the `[Label]` text written onto the rewritten link.
- Extra steps: e.g. speconaut's `prune_main_typ_includes`. A post-plan hook.

Slot rewriting beyond plain `link()` edges is consumer-specific and stays out of
the crate. The crate orchestrates; the consumer supplies eligibility, identity,
label, and any extra edits.

== Scope

- New crate `crates/typst-cleanup`, consumer-blind: no task / MindTape /
  speconaut vocabulary in API or docs.
- Public surface: a `Plan` (deletions + rewrites) built by `plan(hooks)` that
  mints all tombstones and computes all edits before any write; `apply(&Plan)`
  splices + deletes without committing; `dry_run`/`confirm`/report helpers.
  Atomic by construction: any mint or parse failure aborts before a byte changes.
- `mindtape cleanup` becomes a thin consumer: supplies terminal-status
  eligibility + file-stem identity; its bespoke `plan`/`apply`/`report` collapse
  onto the crate.
- speconaut adopts it under a separate speconaut task (its eligibility policy +
  `prune_main_typ_includes` hook).

== Acceptance

`crates/typst-cleanup` builds standalone with no domain vocabulary; tests cover
plan-before-write atomicity, dirty-tree refusal, empty plan, all-or-nothing abort,
multi-survivor link rewrite, and dry-run-no-write; `mindtape cleanup` rides the
crate with no behavioural change (its existing cleanup tests pass unchanged);
`just check` passes.
