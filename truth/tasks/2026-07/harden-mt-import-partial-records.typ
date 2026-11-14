#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Harden mt import against partial canonical records",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 7,
  ),
  tags: ("cli",),
  links: related("canonical-task-json-schema.typ")[schema deserialize gaps]
    + related("task-to-typ-emitter.typ")[dateless-status emission],
  status: done(2026, 7, 13),
)

== Summary

Follow-up from the import milestone. `mt import` works end-to-end (the backlog
round-trips, both jq adapters file valid tasks), but two robustness edges
surfaced during wave 4. We deferred them with documented workarounds rather than
change the schema/emitter contracts mid-integration.

== Findings

- Deserialize is strict. `CanonicalTask` carries `#[serde(default)]` only on the
  framework `score` field, so every non-`Option` key (`id`, `file`, `estimate`,
  `links`, `tags`, `derived_tags`, `body`, `extras`) must appear in the JSON even
  though the docs describe `id`/`file`/`derived_tags` as computed and ignored on
  import. Omitting one is a hard `missing field` parse error, not a default. The
  jq adapters emit every key so they pass, and `www/content/import-export.typ`
  carries a warning callout. A hand-written jq or record is still brittle. Add
  `#[serde(default)]` at the `CanonicalTask` (and nested) level so a producer can
  omit computed/empty fields, matching the "importer ignores them" contract.

- Dateless status vanishes. `emit::emit_status` drops the whole `status:` field
  when `status.date` is `None`, by design, because a dated status is the caller's
  job. A canonical record with a status name but no date then doesn't emit a
  status, and `mt check` fails with "missing required status". `contrib/jq/github.jq` works around
  it with a `createdAt` fallback. The mapper (`import_map`) should instead stamp
  a default date (today) on a name-only status, so the workaround is unnecessary.

== Acceptance

- `#[serde(default)]` lets a canonical record omit computed/empty keys. A
  round-trip and a minimal `{ "title": "..." }`-style record both import.
- A name-only status maps to a dated status without a jq-side fallback.
- Relax the `import-export.typ` warning callout to match.
- `just check` green.

== Scope

- `crates/mindtape/src/canonical.rs` `serde` defaults. `import_map` status-date
  default. The docs callout. Not the jq adapters (they stay correct either way).
