#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Canonical JSON to Task, config policy mapping",
  priority: ice(
    impact: 7,
    confidence: 0.75,
    ease: 4,
  ),
  tags: ("cli",),
  links: parent("import-tasks-from-other-trackers.typ")
    + depends-on("canonical-task-json-schema.typ"),
  status: done(2026, 7, 12),
)

== Summary

This is the config-aware middle layer. It parses a canonical task-JSON record into
a `Task` and reconciles it against this project's `config.toml`. The jq adapter
has already done the structural map (foreign -> canonical, config-blind). This
layer does the policy map (canonical -> valid-for-this-project), the only part
that needs the loaded config. It is pure over `(&Config, canonical JSON)`, with
no `IO` and no source writing. The CLI (`mt-import-command`) drives it and calls
the emitter.

== Design / contract

- New module `crates/mindtape/src/import_map.rs` (name flexible), exported from
  `lib.rs`. Input: a `CanonicalTask` (from `canonical.rs`) or its JSON `Value`,
  plus `&Config`. Output: a `Task` plus a per-task report of what it mapped,
  dropped, or carried to `x-<ns>`.
- Read the authored side of the schema (the `framework` + `inputs` pair or a
  scalar priority, ignoring `score`, with `tags` rather than `derived_tags` and
  `body`).
  This reverses `CanonicalTask::from_task`.
- Policy checks against config, each producing a report entry rather than a hard
  panic. The CLI decides fail, skip or continue:
  - status: must hit the legend, with a `--map old=new` style override and a
    default initial status for a missing status.
  - priority: strict-framework projects (`allow-manual = false`) require the
    framework form. The loader reports scalar input and bounds violations.
  - kind: must hit the enum when the project declares one.
  - tags: filter and report against the allow-list, with an option to
    drop-unmapped.
  - unmapped fields: the jq author decides structural drops. The loader checks
    anything the source carried into an `x-<ns>` slot for `x-` discipline only.
- `body` guarantee: if the record's `body` is empty but the project's
  body-header rules require a `Summary`, synthesize a minimal `== Summary` from
  the title (or a `description`/`body` the jq supplied). This is the policy home
  for the "every file passes `mt check`" invariant, keeping the emitter pure.
- Determinism: stem/slug from the title via `naming` (same as `mt add`). The CLI
  owns dedupe against existing stems, but expose the slug helper here.

== Files

- `crates/mindtape/src/import_map.rs` (+ tests), `lib.rs` export.

== Acceptance

- `just check` green.
- Tests: a canonical record with framework priority -> `Task` with the framework
  preserved. An out-of-legend status -> reported, not silently accepted. An
  out-of-allow-list tag -> reported or dropped per option. A record with no body
  under a Summary-requiring config -> a synthesized `== Summary`.
- Commit `Refs: truth/tasks/canonical-json-to-task-mapping.typ`.

== Scope

- Parsing + config policy mapping + report + `body` guarantee.
- Not in scope: JSON schema definition (done upstream), the emitter, the CLI
  wiring, jq files.
