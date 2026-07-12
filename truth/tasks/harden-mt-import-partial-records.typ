#import "@local/mindtape:0.2.0": *

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
  status: proposed(2026, 7, 13),
)

== Summary

Follow-up from the import milestone. `mt import` works end-to-end (the backlog
round-trips, both jq adapters file valid tasks), but two robustness edges were
found during wave 4 and deferred with documented workarounds rather than
changing the schema/emitter contracts mid-integration.

== Findings

- Deserialize is strict. `CanonicalTask` carries `#[serde(default)]` only on the
  framework `score` field, so every non-`Option` key (`id`, `file`, `estimate`,
  `links`, `tags`, `derived_tags`, `body`, `extras`) must appear in the JSON even
  though `id`/`file`/`derived_tags` are documented as computed and ignored on
  import. Omitting one is a hard `missing field` parse error, not a default. The
  shipped jq adapters emit every key so they pass, and `www/content/import-export.typ`
  carries a warning callout — but a hand-written jq or record is brittle. Add
  `#[serde(default)]` at the `CanonicalTask` (and nested) level so a producer can
  omit computed/empty fields, matching the "importer ignores them" contract.

- Dateless status vanishes. `emit::emit_status` drops the whole `status:` field
  when `status.date` is `None` (by design — a dated status is the caller's job),
  so a canonical record with a status name but no date emits no status and fails
  `mt check` with "missing required status". `contrib/jq/github.jq` works around
  it with a `createdAt` fallback. The mapper (`import_map`) should instead stamp
  a default date (today) on a name-only status, so the workaround is unnecessary.

== Acceptance

- `#[serde(default)]` lets a canonical record omit computed/empty keys; a
  round-trip and a minimal `{ "title": "..." }`-style record both import.
- A name-only status maps to a dated status without a jq-side fallback.
- The `import-export.typ` warning callout is relaxed accordingly.
- `just check` green.

== Scope

- `crates/mindtape/src/canonical.rs` serde defaults; `import_map` status-date
  default; the docs callout. Not the jq adapters (they stay correct either way).
