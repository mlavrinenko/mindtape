#import "@local/mindtape:0.2.0": *

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
  status: proposed(2026, 7, 13),
)

== Summary

The config-aware middle layer: parse a canonical task-JSON record into a `Task`
and reconcile it against this project's `config.toml`. The jq adapter has already
done the structural map (foreign -> canonical, config-blind); this layer does the
policy map (canonical -> valid-for-this-project) — the only part that needs the
loaded config. Pure over `(&Config, canonical JSON)`, no IO, no source writing;
the CLI (`mt-import-command`) drives it and calls the emitter.

== Design / contract

- New module `crates/mindtape/src/import_map.rs` (name flexible), exported from
  `lib.rs`. Input: a `CanonicalTask` (from `canonical.rs`) or its JSON `Value`,
  plus `&Config`. Output: a `Task` plus a per-task report of what was mapped,
  dropped, or carried to `x-<ns>`.
- Read the authored side of the schema (`framework` + `inputs`, or scalar
  priority — ignore `score`; `tags` not `derived_tags`; `body`). Reverse of
  `CanonicalTask::from_task`.
- Policy checks against config, each producing a report entry rather than a hard
  panic; the CLI decides fail vs skip vs continue:
  - status: must hit the legend; a `--map old=new` style override and a default
    initial status for a missing status.
  - priority: strict-framework projects (`allow-manual = false`) require the
    framework form; scalar in / bounds violations reported.
  - kind: must hit the enum when the project declares one.
  - tags: filter/report against the allow-list; option to drop-unmapped.
  - unmapped fields: the jq author decides structural drops; anything the source
    carried into an `x-<ns>` slot is validated for `x-` discipline only.
- `body` guarantee: if the record's `body` is empty but the project's
  body-header rules require a `Summary`, synthesize a minimal `== Summary` from
  the title (or a `description`/`body` the jq supplied). This is the policy home
  for the "every file passes `mt check`" invariant, keeping the emitter pure.
- Determinism: stem/slug from the title via `naming` (same as `mt add`); the CLI
  owns dedupe against existing stems, but expose the slug helper here.

== Files

- `crates/mindtape/src/import_map.rs` (+ tests), `lib.rs` export.

== Acceptance

- `just check` green.
- Tests: a canonical record with framework priority -> `Task` with the framework
  preserved; an out-of-legend status -> reported (not silently accepted); an
  out-of-allow-list tag -> reported/dropped per option; a record with no body
  under a Summary-requiring config -> a synthesized `== Summary`.
- Commit `Refs: truth/tasks/canonical-json-to-task-mapping.typ`.

== Scope

- Parsing + config policy mapping + report + `body` guarantee.
- Not in scope: JSON schema definition (done upstream), the emitter, the CLI
  wiring, jq files.
