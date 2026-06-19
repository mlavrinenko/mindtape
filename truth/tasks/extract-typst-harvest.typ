#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Extract typst-harvest into its own repository",
  status: proposed(2026, 6, 14),
  priority: 5,
  estimate: (hours: 6),
  relates-to: (link("converge-speconaut.typ"),),
  tags: ("substrate",),
)

== Summary

`crates/typst-harvest` is package-agnostic and consumer-blind by construction:
an eval-only Typst `World` plus `metadata()` marker harvesting into a Typst-free
`HVal` tree. Lift it to its own repo so both MindTape and speconaut depend on it
as a shared substrate.

== Scope

- Move the crate out; publish or path/git-depend from MindTape.
- Reconcile with speconaut's existing Typst eval engine (single substrate).
- Keep it harvest-only (read side): no task concepts, no editing.

Follow-up to `docs/decisions/002-per-file-rewrite-and-speconaut-convergence.md`.
