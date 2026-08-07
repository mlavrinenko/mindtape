#import "@local/mindtape:0.2.0": *

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
  status: proposed(2026, 8, 8),
)

== Summary

\`--dedup\` is silently inert when \`\[tasks.naming\] stem\` is set to a template. \`output\_stem\` calls \`naming::disambiguate\` at mint time, so a templated stem never collides and never reaches the new dedup step — \`--dedup fail\` and \`--dedup skip\` both behave as \`add-suffix\`. A flag that does nothing under a configuration, without saying so, is worse than a missing flag. It also breaks the quality-signal sweep, whose whole re-run guard rests on \`skip\` meaning skip: under a templated stem the sweep would file a suffixed duplicate on every run instead of skipping.

== Scope

- Unify the two paths so the dedup mode is the single decision about a collision, whatever produced the stem.
- \`add-suffix\` stays the behaviour a templated stem gets today, so a project that configured \`\[tasks.naming\] stem\` and said nothing about dedup sees no change — make that the config default for that case rather than a special path through \`output\_stem\`.
- \`fail\` and \`skip\` must then work identically with and without a stem template.
- Cover the matrix: each dedup mode crossed with templated and bare stems.
- This was found at integration, not by a test: the existing suite pinned \`output\_stem\`s behaviour and the new suite pinned the dedup step, and nothing exercised both.
