#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Import from other trackers (canonical JSON + jq adapters)",
  priority: ice(
    impact: 7,
    confidence: 0.75,
    ease: 4,
  ),
  tags: ("cli",),
  status: done(2026, 7, 12),
)

== Summary

Milestone (parent of the import work). Give MindTape a migration on-ramp without
taking on a per-source parsing burden. One Rust command, `mt import`, reads a
single canonical task-JSON schema and writes native `.typ` tasks. Foreign
trackers are handled outside the binary by jq programs that transform their own
export into that canonical schema. `mt import` is config-aware: it maps the
canonical record onto this project's status legend / priority scale / kind enum /
tag allow-list, validates every file (`mt check`), and reports mismatches.

The canonical schema is the authored core of `mt ls --json`, so import and list
share one contract and `mt ls --json | mt import` round-trips. Import is
"`mt add` without a template, from a full record" — it reuses the same
validate-before-land guarantee and needs one new primitive: a `Task -> .typ`
emitter.

== Architecture

Three layers, one config-aware middle:

```
foreign export --jq--> canonical JSON --mt import--> Task --emit--> .typ --> mt check
                (contrib, config-blind)  (config-aware map)  (shared)   (existing gate)
```

- jq adapter (contrib/jq, no Rust): structural map foreign->canonical. Ships in
  the repo, versioned against the schema, golden-tested. A contributor adds a
  source with a `.jq` and zero Rust, zero release.
- `mt import` (Rust, one code path): policy map canonical->this-project config
  (legend / scale / kind / tags, unmapped -> report / skip / override), then emit
  + validate. This is the only layer that needs the loaded `config.toml`, which
  jq cannot see.
- emitter (Rust, shared): `Task -> task.with(...)` source. New primitive; the
  only thing a future export would ever reuse.

== Decisions (settled — do not relitigate)

- No per-source Rust adapters. Sources are jq files only. Anything else is a
  maintenance burden this project will not carry.
- No `mt import --via <source>` convenience wrapper. Not now, not later. The
  binary reads canonical JSON on stdin/file; jq is a documented pipe.
- No foreign-format export command. Export is already solved: tasks are plain
  files in git, and `mt ls --json` is lossless. Foreign export = a reverse `.jq`
  a user writes against `ls --json`. Ship a docs cookbook, not code.
- todo.txt is not a first-class source (it is line format, not JSON). A `jq -R`
  recipe in the docs covers it; do not build a line parser.
- Pre-release: refactor freely. No released schema or output to preserve. Break
  and reshape `ls --json` as needed to reach parity.

== Task graph

Children (each one session; depends-on encodes the wave order):

- `canonical-task-json-schema` — define the schema, add `body`, fix priority
  parity; reshape `ls --json` to it. Foundational.
- `task-to-typ-emitter` — the `Task -> .typ` primitive.
- `canonical-json-to-task-mapping` — canonical JSON -> Task, config policy map.
- `mt-import-command` — the `mt import` CLI; round-trip acceptance.
- `jq-source-adapters` — Taskwarrior + GitHub jq files, golden + e2e tests.
- `import-export-docs` — schema reference + import/export cookbook.

== Scope

- The six children above, integrated and gated on merged HEAD.
- `mt ls --json | mt import` round-trips the backlog (field-level + body).
- `task export | jq -f contrib/jq/taskwarrior.jq | mt import` files valid tasks.
- No new runtime dependency in the binary; jq stays a dev/test + docs concern.

== Follow-ups

Delivered in full; the marquee round-trip and both jq adapters pass. Two minor
import-robustness edges surfaced in wave 4 and were deferred with documented
workarounds (not blockers): strict `CanonicalTask` deserialize (all non-`Option`
keys required) and `emit_status` dropping a dateless status. Both are captured in
`harden-mt-import-partial-records.typ`.
