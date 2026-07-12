#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Canonical task-JSON schema (ls/import parity)",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 5,
  ),
  tags: ("cli",),
  links: parent("import-tasks-from-other-trackers.typ"),
  status: proposed(2026, 7, 13),
)

== Summary

Foundational. Define one canonical task-JSON schema, shared by `mt ls --json`
(serialize) and `mt import` (deserialize), so the two never drift and
`mt ls --json | mt import` round-trips. Today the JSON is built ad hoc in
`crates/mindtape-cli/src/output.rs::task_to_json` and is lossy for a write path
in two ways: framework priority collapses to its numeric score, and there is no
`body`. Fix both, and centralize the schema as one serde type in the domain lib.

== Design / contract

- New module `crates/mindtape/src/canonical.rs`: a `CanonicalTask` serde struct
  that is the schema of record, plus `CanonicalTask::from_task(&Task, &Config)`.
  Re-export from `lib.rs`. `output.rs::task_to_json` becomes a thin call into it
  (or is deleted in favour of it) so there is exactly one shape.
- Priority parity: emit the authored form, not just the resolved score. For a
  `NumValue::Framework { name, inputs }` emit e.g.
  `"priority": { "framework": "ice", "inputs": { "impact": 8, "confidence": 0.75,
  "ease": 4 }, "score": 6 }`. `score` is the computed read-side extra (nullable
  outside a project); `framework` + `inputs` are the authored write-side. A
  scalar priority stays a bare number (or `{ "value": n }` — pick one and
  document it). The reverse mapping (`canonical-json-to-task-mapping`) reads the
  authored side and ignores `score`.
- Add `body`: the raw Typst content after the `#show: task.with(...)` line, as a
  string. Emit it in `ls --json`; the emitter writes it back verbatim. This is
  what lets a round-trip preserve the `== Summary` section (so `mt check` passes)
  and what carries a foreign task's description on real imports. Capturing the
  post-marker source tail is the main risk here — the loader currently keeps no
  raw body. If full capture proves larger than one session, scope it: emit
  `body` from what is available and file a follow-up, but keep the field in the
  schema and note the gap in the milestone.
- Additive read-side fields stay (`file`, `id`, `score`, `derived_tags`); the
  importer ignores them. Authored `tags` vs virtual `derived_tags` stay separate
  (already the case) — import reads `tags` only.
- Version the schema: a `SCHEMA_VERSION` const + a short doc comment listing
  every field and whether it is authored (round-trips) or computed (read-only).
  jq authors depend on this contract.

== Files

- `crates/mindtape/src/canonical.rs` (+ `canonical_tests.rs`), `lib.rs` export.
- `crates/mindtape-cli/src/output.rs` — route through `CanonicalTask`.
- Possibly `crates/mindtape/src/loader.rs` / harvest path for body capture.

== Acceptance

- `just check` green (fmt, clippy -D, tests, machete, file-size, jscpd, outdatty).
- Unit test: a framework-priority task serializes with `framework` + `inputs`
  and a scalar-priority task with the scalar form.
- Unit test: a task with a `== Summary` body serializes with `body` non-empty.
- No secrets; commit `Refs: truth/tasks/canonical-task-json-schema.typ`.

== Scope

- Schema module, priority parity, `body` field, `ls --json` reshaped to it.
- Not in scope: the reverse (JSON->Task) mapping, the emitter, the CLI.
