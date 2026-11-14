#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "fill mt add slots from environment variables",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 4,
  tags: ("cli", "substrate"),
  estimate: (files: 3),
  status: done(
    2026,
    8,
    5,
  )[`env` param implemented, e2e proves declared-var fill],
  links: related("assignee-field-core.typ")[
    the assignee slot is this feature's first consumer, and its required splice reads `MINDTAPE_PRINCIPAL` through it
  ],
)

== Summary

`mt add` resolves a template's slots from the explicit inputs (`--set`, `-f`, the
positional title) and the slot default, then falls to the interactive prompt. A
template cannot source a slot from the environment. A value the environment
already carries (an actor's identity, a CI variable, a pinned date) needs a hand
restatement on every task. Let a slot declare an `env` variable and fill from it
when present, skipping that slot's interactive prompt. An absent or unreadable
variable falls through to the normal sources exactly as today.

== Scope

- Extend `slot()` in `lib/prelude.typ` with an `env` param: a string that
  specifies an environment variable, defaulting to none (no environment
  sourcing).
- Fill precedence in the `mt add` path: explicit inputs (`--set` / `-f` /
  positional title) > `env` (present and readable) > slot default > interactive.
  The fill parses an `env` value to the slot's kind the same way it parses an
  explicit `--set` value.
- Discovery reads the environment on the parse plane alongside the other slot
  options (`typst-edit`, no eval). A template then previews without running the
  variable.
- An unset or unreadable variable is not a fill failure. It falls through. The
  same template then works in a bare shell and under a harness that sets it.
- Tests: a slot with `env` fills silently from a set variable, and the
  interactive scaffold skips it. An unset variable leaves the slot to the next source.
- Consumer-blind like the rest of `add_slots`: the reading is generic, and any
  slot can opt in.
