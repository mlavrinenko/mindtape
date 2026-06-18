# 5. MindTape <-> speconaut task-layer convergence boundary

Status: Accepted
Date: 2026-06-18

## Context

ADR 002 set the direction: MindTape becomes a per-file task library plus a thin
CLI, and speconaut depends on it instead of forking a task engine. It left the
exact boundary as a follow-up — which task fields are generic MindTape core and
which stay speconaut-domain, how the two Typst packages relate, and what
speconaut deletes versus keeps. This ADR fixes that boundary so the convergence
tasks (`absorb-task-graph-into-lib`, `core-lifecycle-links`,
`converge-speconaut`, `reviewed-reaffirmation-field`) have a settled contract to
build against.

Ownership split:

- MindTape owns the task model, the `@local/mindtape` DSL, config-driven
  validation, `list` / `show` / `add`, the dependency graph, ranking, and
  `cleanup`. It stays spec-blind and never references speconaut.
- speconaut owns specs<->code (extraction, codegen, audit-traces, diff, lock,
  status) plus the spec-first session workflow. It depends on `crates/mindtape`.

## Decision

1. Core-vs-extras field split. Promote three generic task-lifecycle fields from
   speconaut-only to MindTape core:
   - `supersedes`, `duplicates` — task<->task links, same shape as the existing
     `depends-on` / `relates-to` (link lists of file stems, broken-link
     resolution applies). They are generic lifecycle relations, and SWHID
     cleanup (ADR 003) records them as the reason a retired task was
     tombstoned. Landed by `core-lifecycle-links`.
   - `reviewed` — an optional core date: the last time a non-terminal task was
     re-affirmed as still relevant, a generic backlog-rot signal. Optional, no
     config enum. Landed by `reviewed-reaffirmation-field` (lower priority; may
     lag the two links).

   Everything spec-domain stays behind `x-speconaut`: DDD contexts, decisions,
   spec<->code links, grade semantics speconaut layers on. The test for core is
   "does a generic task tracker want this?" — lifecycle relations and staleness
   yes; spec structure no.

2. One task DSL, two packages, no shim. Tasks are authored with
   `#import "@local/mindtape:0.2.0"` `task()`. The `@local/speconaut` package
   drops `task()` and keeps only its spec constructors (`feature` / `scenario` /
   `decision` / ...). Specs and tasks are two concerns, two packages. There is
   no re-export: a MindTape-only checkout authors, lists, and validates tasks in
   a speconaut worktree unchanged, and a speconaut task stays a valid MindTape
   task. speconaut-only fields ride the `x-speconaut:` slot validated by
   speconaut's own `task-extras()`.

3. speconaut depends on the MindTape library for the typed `Task` model and the
   graph queries (toposort + cycle detection, dependency-ready, `next` ranking),
   per `absorb-task-graph-into-lib`. It deletes its hand-rolled copies rather
   than re-implementing them. The graph API is consumer-blind: it operates on the
   typed model and the configured terminal status set, and knows nothing about
   specs or sessions.

4. The session FSM stays in speconaut. It is spec-first commit enforcement, not
   task tracking, so it does not move to MindTape — but it re-anchors onto a
   MindTape `Task` by id instead of speconaut's own task type.

5. speconaut ships its own `.mindtape/config.toml` (its status lifecycle, its
   bounds, its enum values). MindTape's config schema is the contract; speconaut
   supplies values. MindTape never reads a speconaut-specific key.

## Consequences

- Gain: a settled, generic task substrate; speconaut sheds its task model, its
  task DSL, and its graph code, depending on `crates/mindtape` for all three.
  The shared-checkout property holds in both directions — neither tool's tasks
  are second-class in the other's worktree.
- The marker shape changes when `supersedes` / `duplicates` (and later
  `reviewed`) land. Each promotion bumps `lib/typst.toml`, updates the
  `TYPST_PKG_VERSION` const in `crates/mindtape-cli/src/init.rs`, and the import
  examples in `README.md`, `www/index.html`, and `CLAUDE.md` — tracked by the
  field tasks, not this ADR.
- speconaut keeps the spec packages and the session FSM; only the task layer
  converges. Spec<->code machinery is out of scope here and untouched.
- The speconaut-side migration (delete its task layer, depend on the library,
  re-anchor the FSM) is tracked in the speconaut repo under its own session
  workflow and may lag this ADR.
- This ADR supersedes the convergence note in ADR 002 (point 6 and the
  "Follow-up" consequence): the re-export-shim wording there is replaced by the
  no-shim, two-package decision above. ADR 002's per-file model and extension
  seam otherwise stand.

## Deprecation Criteria

Drop the branch-scoped convergence directive in `CLAUDE.md` (the "Temporary —
active only on branch `redesign/per-file-rewrite`" section) when this branch
merges to `main`; this ADR records that contract durably in its place. Supersede
this ADR if a later one moves a field across the core/extras line, restores a
task DSL to `@local/speconaut`, or relocates the graph API out of
`crates/mindtape`.
