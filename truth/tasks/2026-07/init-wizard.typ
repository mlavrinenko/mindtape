#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "init wizard",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("cli",),
  status: done(2026, 7, 23),
  links: parent("init-presets.typ"),
)

== Summary

A questionnaire over the configuration axes, for someone who does not yet know which preset they want. Each question answers one axis, and the answers compose into one flattened config. A preset is simply a saved answer set over the same axes. That is one model with two entry points, not a second mechanism.

The axes, each one question and one config fragment: workflow (status legend plus transitions), priority, estimate, required fields, organisation (derive-tags), body headers, recurrence.

== Scope

- `mt init` with no `--preset` and a terminal walks the axes. `--preset X` never prompts. `--no-input`, or a non-terminal stdin, falls through to the default preset without asking anything.
- Every question has a corresponding flag, and a flag really suppresses the question it answers. This is the most common bug in the category. npm, create-next-app, create-vite and create-astro all offered a `--yes` or an explicit flag that still prompted, and two of those issues closed unfixed.
- Ask two or three core questions, then offer to accept defaults for the rest. Seven prompts is a wall for someone who wants to start working.
- The wizard records the answers as a comment block in the written config. The project then documents its own construction, and `preset-save` has something to read.
- Coherence, not just composition: the wizard must not permit an answer set whose config the scaffolded templates cannot satisfy. A strict framework priority with no matching template is the concrete case: `required_core_slots` cannot splice a framework that lacks a scalar form. Either synthesise the template or refuse the combination with a message that identifies the conflict.
- Extend the e2e matrix to cover every reachable answer set, not only the four named presets. The matrix already caught one preset requiring a field that its own template could not fill. Free composition multiplies that risk.
- Tests: a TTY-less run never prompts. `--no-input` never prompts. Each flag suppresses its question. An incoherent answer set fails with a useful message. The recorded answer comment round-trips.
