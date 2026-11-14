#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "external artifact links",
  priority: ice(
    impact: 7,
    confidence: 0.75,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("cli", "substrate"),
  status: done(2026, 7, 22),
)

== Summary

Let a task link something outside the backlog: the commit, pull request or build that discharges it. Today every `links:` target resolves to a task file stem, so there is no way to record what a done stamp is actually backed by. That gap is what keeps a review gate a vocabulary rather than an enforcement point: the tool can say a task reached done, but not that any artifact exists.

The shape is a per-kind target validator rather than an exception flag. An `external = true` escape hatch would say "this kind is weird". A defaulted `validate` field says "here is the axis every kind sits on", and the existing behaviour becomes its default value rather than a special case.

== Scope

- `[[tasks.links.kinds]]` gains `validate`, defaulting to `"local-task-path"` — exactly today's behaviour, so every existing config is unchanged.
- Bundled validators: `local-task-path` (default), `url` (RFC 3986), `git-sha` (hex, 7 to 64 characters, covering abbreviated SHA-1 through full SHA-256), `uuid`, `regex`, `any`. The named ones cover the common cases so most projects never write a pattern. `regex` with a sibling `regex = "..."` key is the escape hatch so nobody is stuck. No bare `sha` validator, because it is ambiguous about algorithm and digest length, and that is precisely what `regex` is for.
- Graph semantics follow from the field rather than from a second flag: any kind whose `validate` is not `local-task-path` drops out of stem resolution, cardinality-against-tasks, cycle detection and readiness. Note rules still apply, so a kind can demand a note.
- A malformed target raises a validation error with the kind, the validator and the offending value.
- `mt ls` and `mt graph` render non-task targets distinctly and never try to follow them.
- Tests: each bundled validator accepts a good value and rejects a bad one. A non-task target passes stem resolution and stays out of every cycle. `regex` without a `regex` key is a config error. A note rule still applies to a non-task kind. Graph render omits them from traversal.
