#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Converge task tracking with speconaut",
  status: proposed(2026, 6, 14),
  priority: "normal",
  relates-to: (link("extract-typst-harvest.typ"),),
  tags: ("speconaut",),
)

== Summary

speconaut depends on the MindTape library, ships its own `.mindtape/config.toml`,
re-exports `task` plus the status shortcuts, and passes its fields through the
`x-speconaut:` extension slot. MindTape never references speconaut.

== Scope (lands in the speconaut repo, session-gated)

- Amend speconaut's task-header standard: header gains `x-<ns>`, the field
  allowlist relocates to a `task-extras()` validator.
- Record the convergence-boundary decision; depend on the MindTape library.
- speconaut's `truth/tasks/*.typ` become MindTape task files.

Follow-up to `docs/decisions/002-per-file-rewrite-and-speconaut-convergence.md`.
