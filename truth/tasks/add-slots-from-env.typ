#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Fill `mindtape add` slots from MINDTAPE_ADD_* env vars",
  status: done(2026, 6, 23),
  priority: ice(impact: 4, confidence: 0.7, ease: 7),
  relates-to: (
    link("add-title-from-stdin.typ"),
    link("add-interactive-fill.typ"),
  ),
  tags: ("cli",),
)

== Summary

Let `mindtape add` read slot values from the environment:
`MINDTAPE_ADD_TITLE="hello world" mindtape add` fills the `title` slot. Useful
for CI, wrappers, and shells that prefer env over flags. Mirrors `--set`
semantics: an env value is a string, coerced to the slot's kind by the existing
`fill` render (so `MINDTAPE_ADD_PRIORITY=7` parses as the int slot).

== Scope

- Map per discovered slot, not by scanning the environment: for each `SlotSpec`
  compute `MINDTAPE_ADD_<NAME>` (uppercase, `-` to `_`) and read it. Slot-to-env
  is unambiguous and reads only known slots; unknown env keys are ignored.
- Precedence: `--set` > `-f` > `MINDTAPE_ADD_*` env > stdin title > default. Env
  fills only slots not already provided by `--set`/`-f`. Layer before the
  interactive `should_open` check so an env-filled required slot suppresses the
  auto-editor.
- Value shape mirrors `--set`: `RawValue::Str`, coerced per kind in `fill`.
  Scalars only; lists/multi need `-f` JSON (documented).
- Scope is slots only — `--name`/`--status`/`--into` are not slots and are not
  read from env in this task.
- Pure, hermetic helper `layer_env(values, specs, lookup)` taking an injectable
  lookup closure so tests avoid process-global `std::env`.

== Notes

- `MINDTAPE_ADD_` per-command prefix matches the `[add]` config namespace; no
  existing `MINDTAPE_*` vars to collide with.
- Touches `crates/mindtape-cli/src/add.rs` (run) and `add_values.rs` (helper +
  tests). No library churn.
- README/`add` docs gain the env-var form.
