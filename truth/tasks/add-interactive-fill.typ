#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Interactive editor fill for `mindtape add`",
  status: wip(2026, 6, 22)[Editor-buffer fill shipped (YAML, block scalars for `text` slots, parsed back via `serde_yaml_ng`, TTY-gated, editor ladder `[add] editor` → `$VISUAL` → `$EDITOR` → `vi`). Scaffold refinement pass: barebones built-in template (just a required `title` slot), config-required core fields spliced in as slots so they surface even with a barebones template, scaffold tail trimmed (jump-to-end lands on the last fillable line), and a customizable `[add] header` (custom text auto comment-prefixed, `""` disables).],
  priority: ice(impact: 6, confidence: 0.7, ease: 6),
  tags: ("cli",),
)

== Summary

`mindtape add` is built for machines first: slots are filled from `-f values.json`,
`--set k=v`, or defaults, and a bare `mindtape add` hard-errors with
`missing required slot(s): title, summary`. For a human filing a task by hand
this is a dead end — they must already know every slot name and its kind. Give
humans an interactive fill: open their editor on a pre-scaffolded buffer holding
one entry per slot (annotated with the slot's prompt, kind, default, choices),
let them fill it, then run the existing fill + validate pipeline. Works for any
template, since slots are discovered per template (`discover_slots`).

The machinery is already 90% there. `add_values::collect_values` parses values
into a `serde_json::Map` and coerces to `RawValue` via `json_to_raw`;
`--describe=json` already emits the slot schema (name, kind, required, default,
prompt, choices). The interactive buffer is just an editable values file built
from that schema and parsed back through the same coercion. All of it lives in
`crates/mindtape-cli`; the `mindtape` library stays consumer-blind and
JSON/format-free per the convergence directive — no library churn.

== Scope

- Buffer format: YAML. Chosen for block scalars (`summary: |-`) — `text` slots
  (Summary, Scope) are multi-line Typst markup, and YAML block scalars edit those
  far more naturally than TOML's quoting or JSON's escaped one-liners. Comments
  above each key carry the slot's prompt / kind / required / choices / default.
  YAML scalars and sequences map 1:1 onto `RawValue` exactly as JSON does. The
  one cost — `serde_yaml` is archived (unmaintained since 2024-03) — is paid by
  pinning a maintained drop-in fork (`serde_yaml_ng`), used for parse-back only;
  the scaffold is hand-written so it controls the comments and block scalars.

- Trigger semantics. Do NOT auto-open the editor on bare `mindtape add`. `add` is
  also a programmatic/agent surface and `missing required slot(s): …` is a
  machine contract; auto-opening would hang in CI / non-TTY. Gate: open
  interactively only when stdin/stdout is a TTY AND the user opted in (an
  `--edit` / `--interactive` flag, or config `interactive = "auto"`). Non-TTY
  keeps today's exact hard error.

- Editor detection. Ladder: config `[add] editor` → `$VISUAL` → `$EDITOR` →
  fallback (`vi`/`nano`). Either the small purpose-built `edit` crate (tempfile +
  `$VISUAL`/`$EDITOR` spawn) or ~40 lines hand-rolled over `std::process::Command`
  + `tempfile` (already a dev-dep; promote to a real dep). Lean hand-rolled to
  keep deps minimal.

- Scaffold + round-trip. Build the buffer from the discovered `SlotSpec`s
  (pre-fill defaults so the user edits in place; annotate prompt/kind/choices as
  comments). Parse the saved buffer back into `BTreeMap<String, RawValue>`
  (TOML → `serde_json::Value` reuses the existing `json_to_raw`, or a small
  `toml_to_raw` mirror). Feed the existing `fill` → `inject_header` → write →
  `verify` path unchanged.

- Scaffold shape (refinement). The built-in template is barebones — a single
  required `title` slot, no `Summary`/`Scope` body. Core fields the project
  config requires (priority, difficulty, kind, estimate) are spliced into
  `task.with(…)` as `slot(...)` calls before discovery
  (`mindtape::required_core_slots`, `crates/mindtape/src/add_slots.rs`), so a
  required field still surfaces in the scaffold, `--describe`, and the written
  task. Strict-framework priority (no scalar form) is not spliced.

- Buffer ergonomics. The scaffold tail carries no trailing blank line and empty
  scalar entries end `key: ` (trailing space), so an editor's jump-to-end lands
  on the last fillable value position. The instruction header is customizable via
  `[add] header`: custom text (auto comment-prefixed to stay valid YAML) or `""`
  to drop the header entirely; absent keeps the built-in instructions.

- Validation loop (nice-to-have, phase 2). After the editor closes, run fill +
  validate; on failure re-open the buffer with the error annotated as a comment
  rather than aborting.

- Config surface. New `[add]` section (mirrors the existing per-command `[rm]` /
  `[query]` / `[templates]` sections): `editor` (override) and `interactive`
  (`"auto"` | `"always"` | `"never"`). Threads through `Config` in
  `crates/mindtape/src/config.rs`.

== Notes

- Alternative UX considered: inline field-by-field prompts (`dialoguer` /
  `inquire`) — `choice` → select menu, `bool` → confirm, `date` → format-checked.
  Friendlier for newcomers and validates per field, but a heavier TUI dependency,
  a different interaction model, and it does not batch. Treat as a possible later
  addition, not the first cut. Editor-buffer is the lower-risk start because it
  reuses the whole existing pipeline.

- Dependencies on the chosen path: `serde_yaml_ng` (maintained `serde_yaml`
  fork) + `tempfile` added to `crates/mindtape-cli`. Avoid archived `serde_yaml`.
  Editor spawn is hand-rolled over `std::process::Command`, no `edit` crate.

- Touches `crates/mindtape-cli/src/add.rs`, `add_values.rs`, and
  `crates/mindtape/src/config.rs` (the `[add]` section). No change to template
  discovery, slot model, or `fill`.
