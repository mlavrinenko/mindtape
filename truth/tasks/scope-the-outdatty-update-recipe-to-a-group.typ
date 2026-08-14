#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "scope the outdatty update recipe to a group",
  priority: ice(
    impact: 6,
    confidence: 0.85,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (files: 3),
  status: proposed(2026, 8, 14),
)

== Summary

`just outdatty-update` runs a blanket `outdatty update`, which re-records every
group's dependents — including groups whose source nobody touched, and whose
dependents nobody reviewed.

== What a recorded dependent hash means

It is the state of the dependent at the last time that group's source changed
and someone reviewed it. That is why a stale-looking hash is not drift: it is
the review watermark. `outdatty check` re-verifies a group's dependents only
when the group's own source hash moved, which is exactly right.

Blanket re-recording destroys the watermark. It writes "reviewed at this state"
for a review that never happened.

== Observed, on this repo, today

Reconciling a Justfile-only change with a blanket update reported:

```
updated  prelude-docs
updated  cli-name
```

Neither group's source was touched. `docs/src/agents.typ` is a dependent of
`dev-docs`, `cli-name` and `prelude-docs`; `dev-docs` had it recorded at its
current hash, the other two at an older one. The blanket update would have
advanced all three to current, silently claiming that the `lib/prelude.typ` and
`cli.rs` dependents had been reviewed against their sources. They had not.

The stale hash occurs twice in `outdatty.lock` right now. That is the mechanism
working, not a bug to clean up.

== Scope

- `outdatty-update` should take the group as an argument rather than defaulting
  to everything. With the `arg` attribute work landed, that is a parameter with
  a `help` string and a `--usage` entry.
- Keep a way to update all groups deliberately, but make it the explicit choice,
  not the default reachable by muscle memory.
- `CONTRIBUTING.md` tells contributors to run `just outdatty-update` after
  editing a source. That sentence has to change with the recipe — it is a
  `dev-docs` dependent, so the drift gate will say so.
- This was found while delegating a Justfile edit: the agent reached for
  `outdatty update --group dev-docs` on its own and was right to. The recipe
  should not require that judgement from a first-time contributor.
