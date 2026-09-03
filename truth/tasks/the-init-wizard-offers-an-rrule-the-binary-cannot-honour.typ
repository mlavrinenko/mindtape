#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "the init wizard offers an rrule the binary cannot honour",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("cli",),
  estimate: (files: 4),
  assignee: "agent",
  status: done(2026, 9, 3)[just check and just check-rrule green],
)

== Summary

\`mt init --recurrence rrule\` writes \`\[tasks.recur\] rrule = true\` from any
build, but the expansion engine is behind the \`rrule\` cargo feature and
\`just install-bin\` builds without it. The config is accepted, so the failure
surfaces later and elsewhere: the first task authored with \`recur:
rrule("FREQ=MONTHLY")\` fails \`mt check\` with "requires mindtape to be built
with the \`rrule\` cargo feature (not compiled into this binary)". Reported
from a real project, which lost the answer to a re-file with \`every(months:
1)\` and a comment in its own config warning the next reader off the wizard's
own menu entry. A wizard is a promise that the composed config runs; offering
a value this binary cannot act on breaks it at the only moment the user
cannot check.


== Scope

- an axis choice declares whether THIS build can honour it; unavailable by
  default means nothing, so the six other axes are untouched
- the interactive menu lists only honourable choices, and renumbers around a
  hidden one rather than leaving a gap
- an unavailable slug typed at the prompt is refused by name and reason, not
  with the generic "not one of the choices"
- \`--recurrence rrule\` on a build without the feature fails at \`mt init\`,
  naming the cargo feature, instead of writing a config that fails later
- an axis left with a single honourable choice is not asked at all
- the \`rrule\` build keeps every choice: \`just check-rrule\` covers that arm

