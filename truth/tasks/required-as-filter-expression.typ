#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "required as filter expression",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("substrate", "cli"),
  status: wip(2026, 7, 22),
)

== Summary

Replace the bespoke `required` vocabulary with the filter language the rest of the config already speaks. Today `[tasks.priority] required` takes `true`, `false` or the magic string `"non-terminal"` — a one-off enum that cannot express anything else. Making it `true`, `false` or a filter expression turns `"non-terminal"` into `"!terminal"` and opens every other condition for free: required only on a `kind`, only inside a view, only when another field is set.

This is a breaking config change and is deliberately taken now, while the only consumer is this repository.

== Scope

- `required` accepts `true`, `false`, or a filter expression string, uniformly across `priority`, `difficulty`, `kind` and `estimate`.
- No compatibility shim for `"non-terminal"`. It parses as a filter expression, fails as an unknown identifier, and the error names the replacement. A silent alias would leave two spellings in the wild forever.
- The expression is evaluated against the task being validated, with the same evaluation context `mt check` already uses for filters.
- A malformed expression is a config error at load time, not a per-task validation error — one clear failure rather than one per artifact.
- Migrate the shipped configs: the embedded floor, the four `mt init` presets, and this repository's own `.mindtape/config.toml`.
- No `not(...)` function is added. The filter language already has `!`, and a second spelling earns nothing; the shell-quoting friction that motivated it is solved by single-quoting the argument.
- Tests: `true` and `false` behave as before; `"!terminal"` matches the old `"non-terminal"` behaviour exactly; a field required only for one `kind`; a malformed expression fails at config load naming the field; the literal `"non-terminal"` produces an error pointing at `"!terminal"`.
