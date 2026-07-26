#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Reject unknown mt add --set keys",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 2),
  links: related("correct-the-mindtape-skill-against-the-cli.typ")[
    found while correcting the agent skill against the real CLI
  ],
  status: proposed(2026, 7, 26),
)

== Summary

mt add silently drops a --set key that names no slot of the active template: mt add fix bug --set priority=3 exits 0 and writes the template default priority, not 3. The caller gets a success report and a task carrying the wrong value, with nothing in the output to notice. Contrast a list-slot misuse (--set tags=a,b), which is already a hard error naming the fix — unknown keys should fail the same way.

== Scope

- Error on any --set / -f values key that matches no discovered slot, listing the known slot names (mirroring the existing list-slot error).
- Same treatment for the -f/--values JSON path.
- Regression test per entry point; check the interactive --edit scaffold path is unaffected.
