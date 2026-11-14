#import "@local/mindtape:0.3.0": *

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
  status: done(
    2026,
    7,
    26,
  )[landed the `--set`/`-f` unknown-slot-key hard error],
)

== Summary

mt add silently drops a `--set` key outside the active template's slots. The command `mt add fix bug --set priority=3` exits 0 and writes the template default priority, not 3. The caller gets a success report and a task carrying the wrong value, with nothing in the output to notice. Contrast a list-slot misuse (`--set tags=a,b`), which already fails with a hard error that states the fix. Unknown keys should fail the same way.

== Scope

- Error on any `--set` / `-f values` key outside the discovered slots, listing the known slot names (mirroring the existing list-slot error).
- Same treatment for the `-f`/`--values` JSON path.
- Regression test per entry point. Check that the interactive `--edit` scaffold path stays unaffected.
