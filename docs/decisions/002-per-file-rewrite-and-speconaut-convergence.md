# 2. Per-file rewrite, no daemon, and speconaut convergence

Status: Accepted
Date: 2026-06-14

## Context

MindTape today stores tasks as inline checkbox markers
(`- [ ] text #due #high #id`) that a `watch` daemon evaluates and indexes into a
global SQLite database at `~/.local/share/mindtape/index.db`. The operator
dislikes the inline-marker shape and wants the per-file artifact shape of the
sibling tool speconaut: one task per file, `#show: task.with(...)`, filename
identity, a typed status lifecycle.

Two trackers in one portfolio duplicate machinery — two Typst eval Worlds, two
slot rewriters. speconaut also wants a generic task substrate it can extend with
its own fields (contexts, grade, estimate, ...) without forking a task engine.
MindTape wants to shed the daemon and the global index and run per-project.

A shape probe (`docs/redesign/probe`) rewrote 5 real speconaut tasks into the
proposed shape; all compile on Typst 0.14.2, and both misuse cases are rejected
at compile time.

## Decision

1. One task per file via `#show: task.with(...)`. Drop inline checkbox lists and
   multi-task files.
2. No daemon, no SQLite, no global index, no cross-project agenda. Re-evaluate
   the configured task roots live per command, project-wise (matches speconaut).
   Remove `watch`, the `mindtape-store` crate, eval-hash skipping. Operator
   confirmed no global agendas are wanted.
3. Identity is the filename stem; links between tasks break loud at compile time
   on rename. Drop inline `#id()`.
4. Generic configurable core fields: title, status, priority, grade, kind,
   estimate-hours, depends-on, relates-to, due, start, tags, id. Allowed enum
   values, required-unless-terminal, and number bounds come from
   `.mindtape/config.toml`; omit to leave a field unconstrained. Status is a
   `(name, date, note)` dict; config lists the legal names and the terminal
   subset; the prelude ships a broad shortcut set used directly when
   unconfigured.
5. Extension seam: `task` accepts any `x-<ns>:` named slot (opaque value) and
   rejects unknown non-x- slots. MindTape stays consumer-blind. Each consumer
   ships its own `<ns>-extras()` validator (closed allowlist) and reads its
   payload back. The filter language flattens extras to path-addressed
   properties.
6. MindTape becomes a library plus a thin CLI. speconaut depends on the library,
   ships its own `.mindtape/config.toml`, re-exports `task` + status shortcuts,
   and passes its fields as `x-speconaut: task-extras(...)`. MindTape never
   references speconaut or its layout.
7. Keep the filter-expression query language; it now runs over live-evaluated
   items rather than SQL.

Validation splits three ways: the prelude enforces structure (one task, x-
discipline) and stays config-free; the MindTape Rust layer enforces
config-driven enums / required / bounds, identity, query, mutation; each consumer
validates its own extension fields.

## Consequences

- Lose: global cross-project agenda, `watch`, the SQLite store and its
  migrations, skip-on-unchanged hashing, inline multi-task files, inline UUID
  identity. Filter queries re-evaluate roots each call — fine at project size,
  no warm cache.
- Gain: speconaut-grade per-file artifacts; a typed, tool-agnostic extension
  seam; one shared Typst eval/edit substrate across both tools; a simpler
  architecture (no daemon lifecycle, no database).
- Migration: existing inline-marker files need one-time conversion to per-file
  artifacts. A converter is in scope for the rewrite.
- Follow-up (separate task, both repos): extract the shared Typst eval World and
  the in-place slot/marker editor into crates reconciled with speconaut's
  engine. speconaut-side work — amend its task-header standard, record the
  convergence boundary, depend on the library — lands in the speconaut repo
  under its own session workflow.

## Deprecation Criteria

Drop this ADR when the rewrite has shipped and the per-file model is the only one
MindTape supports, or supersede it if a later ADR amends the core field model or
restores any removed subsystem.
