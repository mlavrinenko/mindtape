#import "@local/mindtape:0.2.0": *

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
  status: done(2026, 8, 5)[env param implemented; e2e proves declared-var fill],
  links: related("assignee-field-core.typ")[
    the assignee slot is this feature's first consumer, and its required splice reads MINDTAPE_PRINCIPAL through it
  ],
)

== Summary

mt add resolves a template's slots from the explicit inputs (`--set`, `-f`, the positional title) and the slot default, then falls to the interactive prompt. There is no way for a template to source a slot from the environment, so a value the environment already carries (an actor's identity, a CI variable, a pinned date) is restated by hand every task. Let a slot declare an env variable and fill from it when present, skipping that slot's interactive prompt; an absent or unreadable variable falls through to the normal sources exactly as today.

== Scope

- Extend slot() in lib/prelude.typ with an env param: a string naming an environment variable; defaults to none (no env sourcing).
- Fill precedence in the mt add path: explicit inputs (`--set` / `-f` / positional title) > env (present and readable) > slot default > interactive. An env value is parsed to the slot's kind the same way an explicit `--set` value is.
- Discovery reads env on the parse plane alongside the other slot options (typst-edit, no eval), so a template is previewable and never runs the variable.
- Unset/unreadable env var is not a fill failure — it falls through, so the same template works in a bare shell and under a harness that sets it.
- Tests: a slot with env fills silently from a set variable and is skipped from the interactive scaffold; an unset variable leaves the slot to the next source.
- Consumer-blind like the rest of add_slots: the reading is generic; any slot can opt in.
