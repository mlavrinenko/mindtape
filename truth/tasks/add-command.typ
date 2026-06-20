#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "add command with Typst slot templates",
  status: proposed(2026, 6, 20),
  priority: ice(impact: 7, confidence: 0.8, ease: 6),
  tags: ("cli", "substrate"),
)

== Summary

`mindtape add` files a new task from a Typst template. Templates are ordinary
task artifacts whose authored values are `slot(...)` calls; `add` discovers each
slot on the parse plane (`typst-edit::find_calls`), fills it from JSON/`--set`
values or the slot's own default, and rewrites the source so the filed task
holds plain literals and no `slot` call. No eval, no Typst World on the add path.

The flow is built for non-interactive agents: `--describe[=json]` dumps the slot
schema (the exact fill keys), `--values -` takes a merged JSON blob, `--print`
previews before writing.

== Scope

Engine in `crates/mindtape` (reusable; speconaut converges onto it), CLI thin.

- prelude: add `slot(name, default: none, kind: "string", prompt: none,
  required: true, choices: none)` returning `default`. A filed task never calls
  it; at eval time a template stays a previewable, valid task.
- `crates/mindtape/src/template.rs`: `SlotSpec` / `SlotKind`, `discover_slots`
  (over `typst-edit`), `fill(source, values)` with per-kind value rendering
  (`string`→`"…"`, `int`, `float`, `bool`, `date`→`on(Y,M,D)`, `list`→`("a",…)`,
  `text` body block, `choice`). Unfilled + has default → splice the default's
  raw source; unfilled + required → error; markup `#slot(…)` → replace incl `#`.
  Slot args are literals only (v1, no eval for discovery).
- `[templates]` config: `dirs` (template dirs, relative to root) and `default`
  (stem `add` uses when none named). Built-in `task` template baked in.
- add-owned fields (not slots): initial `status` (first non-terminal legend
  value, dated today) injected into `task.with(…)`; `id` generated when
  `[tasks.id] format = uuid-*`. Date-spread problem avoided by keeping status off
  the template.
- CLI `mindtape add [TEMPLATE]`:
  - `--values <path|->` repeatable, merged left-to-right (preset stacking);
    `--set k=v` thin final override.
  - `--into <DIR>` destination; default `.` = first configured root. Explicit
    value is a filesystem path; warn when it lands outside every root.
  - `--name <STEM>` else slug(title); identity collision is a hard error.
  - `--describe[=text|json]`, `--list`, `--print`, `--edit`, `--no-verify`.
  - After write, load+validate the new file.
- Refactor `review`'s named-arg upsert into a shared helper so `add`'s status
  injection reuses it (no jscpd duplication).
