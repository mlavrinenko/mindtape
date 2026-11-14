#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "make dedup modes reachable under a templated stem",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("cli",),
  estimate: (files: 4),
  links: related(
    "add-dedup-modes-to-mt-add.typ",
  )[the task that landed the flag this makes reachable],
  status: done(
    2026,
    8,
    8,
  )[one policy settles a collision however a stem comes out. Verified live: `skip` skips, `fail` errors, an unset default stays `add-suffix` under a stem template],
)

== Summary

`--dedup` goes silently inert when `[tasks.naming] stem` names a template.
`output_stem` calls `naming::disambiguate` at mint time, so a templated stem
never collides and never reaches the new `dedup` step: `--dedup fail` and
`--dedup skip` both behave as `add-suffix`. A flag that does nothing under a
configuration, without saying so, is worse than a missing flag. It also breaks
the quality-signal sweep, whose whole re-run guard rests on `skip` meaning skip.
Under a templated stem the sweep would file a suffixed duplicate on every run
instead of skipping.

== Scope

- Unify the two paths so the `dedup` mode makes the single collision decision,
  whatever produced the stem.
- `add-suffix` stays the behaviour a templated stem gets today, so a project
  that set `[tasks.naming] stem` and said nothing about `dedup` keeps the same
  result. Make that the config default for the case rather than a special path
  through `output_stem`.
- `fail` and `skip` must then work identically with and without a stem template.
- Cover the matrix: each `dedup` mode crossed with templated and bare stems.
- This came up at integration, not in a test. The existing suite pinned
  `output_stem`'s behaviour and the new suite pinned the `dedup` step, and
  nothing exercised both.
