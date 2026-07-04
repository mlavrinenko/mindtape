#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Default terminal legend when `[[tasks.status]]` is omitted",
  status: cancelled(2026, 7, 4)[superseded by config-embedded-defaults (option A)],
  priority: ice(impact: 6, confidence: 0.7, ease: 5),
  tags: ("gating",),
)

== Summary

The status legend serves double duty: it constrains legal status values and
declares which values are terminal (plus display glyph/colour). The config is a
white-list — "omit a section to leave that aspect unconstrained" — but omitting
`[[tasks.status]]` also wipes the terminal set, so `Legend::terminal()` returns
empty. Every terminal-dependent feature then misbehaves: required-unless-terminal
never relaxes (no task is ever terminal), bare `mindtape rm` (default filter
`terminal`) sweeps nothing, and an overdue `done` task gets the red overdue
accent. There is no way today to say "any status allowed, but these end a task."

== Scope

- Decide how terminal semantics survive an omitted legend. Options:
  - A. Baked-in default legend deep-merged into the config doc before
    deserialization, mirroring the priority-framework precedent
    (`merge_builtin_frameworks`, `crates/mindtape/src/framework.rs`): the
    prelude's shortcut set (`proposed`/`accepted`/`wip` non-terminal,
    `done`/`cancelled`/`failed` terminal). Omit `[[tasks.status]]` → sane
    terminal defaults; declare it → full override.
  - B. Separate the terminal declaration from the value allow-list so
    terminal-ness can be set without constraining values.
  - C. Empty-legend fallback: when no legend is declared, terminal-dependent
    features consult a built-in terminal set while values stay unconstrained.
- Whatever the choice, reconcile it with the white-list philosophy and document
  the interaction in `crates/mindtape-cli/src/default-config.toml`.

== Notes

- Recommend option A: consistent with the framework deep-merge layer, keeps the
  prelude shortcuts and the Rust terminal set in agreement, and an explicit
  `[[tasks.status]]` still fully overrides.
- Real configs (the dogfood) declare a legend, so this bites only minimal
  configs — but it is the same root cause behind several features, worth one
  deliberate fix rather than per-feature patches.
- Surfaced while reviewing the overdue-accent feature
  (`ls-accent-overdue.typ`): an overdue terminal task is accented only because
  the empty legend reports no terminal statuses.
