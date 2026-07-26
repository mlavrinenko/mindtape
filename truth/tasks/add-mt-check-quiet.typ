#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Add mt check --quiet",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("cli",),
  estimate: (files: 3),
  status: done(2026, 7, 26),
)

== Summary

mt check always prints a summary line, so every gate that wraps it has to discard stdout to stay quiet. It is used exactly that way already: as an on.flip gate action, in just check-examples, and in pre-commit hooks, where the only signal wanted is the exit code and the diagnostics on failure. Peer tools take -q for this (linecop --quiet, jscpd), and mt has no quiet form at all — only -v/--verbose in the other direction.

== Scope

- Add -q/--quiet to mt check: suppress the ok summary, keep violations and the exit code.
- Decide whether it belongs on the global flag list rather than on check alone — mt add, flip and rm all print a success line that a script has no use for.
- Regression test that a clean run is silent and a failing run still reports.
