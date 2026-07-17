#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Emit typstyle-clean Typst on write",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  tags: ("cli", "tooling",),
  status: wip(2026, 7, 17)[implementing typstyle-core formatting at write boundaries],
)

== Summary

`mt` writes task files by surgical means only: `typst-edit` byte splices
(`mt flip` / `mt review` / `mt recur`), parse-plane slot fills (`mt add`), and
`emit_task` (`mt import`). Every path preserves the surrounding whitespace
verbatim and never reformats, so a fill or an injected argument can leave
non-canonical indentation behind. Observed recurring across sessions: a task
body carrying ~400 spaces of leading indentation from the original `mt add`
write. It slips every gate — nothing formats task files in CI — yet it is ugly,
and the same class keeps recurring.

typstyle strips exactly this drift (deep body indentation, list nesting), and it
ships a library crate. So format the source `mt` writes through typstyle-core
in-process, rather than shelling out.

== Decisions

- typstyle-core as a library, not an external formatter command. It is
  deterministic, in-process, idempotent, and error-tolerant (invalid Typst
  returns the input unchanged), which matches `mt`'s pure/byte-for-byte ethos and
  keeps tests hermetic with no runtime binary on `PATH`.
- Version already aligned: typstyle-core 0.15 pins `typst-syntax` 0.15 — the
  exact version `mindtape` already resolves via `typst-harvest` 0.2 — so no
  duplicate `typst` crates enter the tree.
- A `[format] enabled` toggle (default `true`) is the only config surface. It
  lets a project that wants minimal-diff surgical edits opt out. No
  external-command formatter knob: that would be non-deterministic, non-hermetic,
  and an arbitrary-exec surface.
- `format_typst` lives in the `mindtape` library (reusable by consumers, per the
  convergence directive); the CLI wraps it with the toggle at the write boundary.
  `emit_task` stays a pure, unformatted serializer — its byte-for-byte contract
  is unchanged.

== Scope

- Add `mindtape::format_typst` (new `format.rs`) over typstyle-core; export it.
- Add `FormatConfig { enabled }` (new `config_format.rs`; `config.rs` is at the
  file-size ceiling) and a `[format]` section on `Config`.
- Canonicalize at every write/print boundary: `mt add` (write and `--print`),
  `mt flip`, `mt review`, `mt recur` (spawn / mutate / skip / advance-chain),
  `mt import`.
- Regenerate `schema/config.schema.json`; document the toggle.
- Tests: `format_typst` strips deep indentation, is idempotent, and passes
  malformed source through; end-to-end `mt add`/`mt flip` emit clean output and
  the toggle-off path preserves surgical edits.
