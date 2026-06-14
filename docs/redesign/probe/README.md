# MindTape redesign — DSL shape probe

Validates the redesign task DSL (per-file core + `x-<ns>` extension seam, see
`../prelude.typ`) compiles on Typst 0.14.2 and rejects misuse. Uses 5 real
speconaut tasks as a second, speconaut-blind consumer to prove the seam works
for a tool MindTape knows nothing about.

## Files

- `../prelude.typ` — prototype MindTape prelude under test.
- `speconaut-stub.typ` — how speconaut consumes it: re-exports `task` + status
  shortcuts, adds `bounded-context` and the closed-allowlist `task-extras`.
- `contexts-stub.typ` — speconaut domain bounded-contexts.
- `01_minimal` … `05_terminal_adapter` — real speconaut tasks rewritten into the
  new shape (expect compile).
- `90_reject_unknown`, `91_reject_typo_extra` — misuse (expect rejection).

## Run

In the dev shell, from this directory:

```
typst compile --root .. 01_minimal.typ /tmp/out.pdf
```

`--root ..` (the `docs/redesign` dir) lets probe files import `../prelude.typ`
without escaping Typst's project root.

## Results (2026-06-14, typst 0.14.2)

```
PASS      01_minimal 02_relates 03_multi_wip 04_decisions 05_terminal_adapter
REJECTED  90_reject_unknown      unknown task slot `contexts`; use x-<ns>
REJECTED  91_reject_typo_extra   unknown speconaut task field `contxts`
```

## Findings

- `task(body, ..meta)` + an `x-<ns>` named sink is valid Typst; an unknown
  non-x- slot is rejected at compile time.
- speconaut re-exports MindTape's `task`, so task files keep a single import —
  zero import churn across a migration.
- The closed allowlist relocated off `task` onto `task-extras` (consumer-owned)
  plus MindTape's x- assertion. Both rejection paths fire with clear messages.
- Wart, accepted: contexts-only tasks wrap one content line in two ceremony
  lines (`x-speconaut: task-extras( … )`).
