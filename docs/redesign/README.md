# MindTape redesign — per-file, daemon-free, extensible

Status: proposed (2026-06-14). Authoritative decision:
`../decisions/002-per-file-rewrite-and-speconaut-convergence.md`. Prototype
prelude: `prelude.typ`. Config schema: `config.example.toml`. Validation:
`probe/`.

## What changes

MindTape moves from inline checkbox markers indexed into a global SQLite
database by a `watch` daemon, to per-file task artifacts re-evaluated live per
command, project-wise. One task per file via `#show: task.with(...)` — the
artifact shape of the sibling tool speconaut.

Removed: the `watch` daemon, the SQLite index, the `mindtape-store` crate,
eval-hash skip-on-unchanged, the global cross-project agenda, inline `#id()`
identity, multi-task files. Operator confirmed: no global agendas wanted.

Kept: Typst as source of truth, eval-only (`typst_eval::eval`), the
filter-expression query language (now over live-evaluated items, not SQL).

## Task model

Identity is the filename stem; renames break links loud at compile time. Each
file emits one labelled `<mindtape>` metadata marker carrying `fields` + `extras`.

Generic configurable core fields (constraints from `.mindtape/config.toml`):

| field | shape |
|---|---|
| title | string |
| status | `(name, date, note)` dict; configurable set + terminal subset |
| priority, grade, kind | configurable enum; required = true / false / non-terminal |
| estimate-hours | number; optional min |
| depends-on, relates-to | task↔task links |
| due, start | dates |
| tags | multi-valued, optional allow-list |
| id | filename stem |

Status ships a broad shortcut set (draft, proposed, accepted, todo, wip, doing,
blocked, in-review, done, cancelled, failed); unconfigured projects use them
directly.

## Extension seam

`task` accepts any `x-<ns>:` named slot, value opaque, and rejects unknown
non-x- slots. MindTape stays consumer-blind. Each consumer ships its own
`<ns>-extras()` validator and reads its payload back. For its filter language
MindTape flattens extras to path-addressed properties (`x-speconaut.contexts`),
semantics opaque.

## Validation ownership

- Prelude (Typst): structure only — one task per file, x- discipline.
  Config-free, so one package serves every project.
- MindTape Rust, from `.mindtape/config.toml`: enum values,
  required-unless-terminal, number bounds; identity; filter; in-place mutation.
- Consumer (e.g. speconaut `task-extras`): closed allowlist over its own fields.

## Convergence with speconaut

speconaut depends on the MindTape library; MindTape never references speconaut.
speconaut's `truth/tasks/*.typ` become MindTape task files; speconaut re-exports
MindTape's `task` + status shortcuts (existing single import unchanged) and
passes its fields as `x-speconaut: task-extras(contexts, reviewed, decisions,
code, supersedes, duplicates)`. Everything scalar/enum — priority, grade, kind,
estimate-hours, status — is generic core. The filter language is a MindTape
asset speconaut gains.

Shared substrate (separate task, both repos): extract the Typst eval World and
the in-place slot/marker editor into crates both tools consume, reconciling
`mindtape-eval` with speconaut's engine.

## Probe outcome

`probe/` rewrote 5 representative live speconaut tasks (minimal, relates-to,
multi-context wip, decisions, terminal adapter) into the new shape. All compile
on Typst 0.14.2; a misplaced top-level `contexts:` and a typo'd extras field are
both rejected. One wart accepted: contexts-only tasks wrap one content line in
two ceremony lines.

## Sequencing

MindTape: (1) prelude + `.mindtape/config.toml` loader + per-file extractor over
shared eval; (2) port the filter language onto live items; (3) in-place editor
for status/field mutation; (4) migration tool from inline markers; (5) remove
the store crate and `watch`.

speconaut (its own repo, session-gated): amend `standards-and-headers` (task
header gains `x-<ns>`, allowlist relocates to `task-extras` + check); record the
convergence-boundary decision; escalate `converge-task-tracking-with-mindtape`;
depend on the MindTape library.
