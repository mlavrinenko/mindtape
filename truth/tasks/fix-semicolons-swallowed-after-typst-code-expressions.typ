#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Fix semicolons swallowed after Typst code expressions",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (files: 1),
  status: proposed(2026, 7, 23),
)

== Summary

In Typst markup a semicolon written immediately after a code expression is silently consumed and never rendered, so two sentences on the comparison page ship as run-ons in the built HTML. Fix the instances and record the gotcha so it does not recur.

== Scope

- Rewrite the affected sentences so no semicolon directly follows a code expression
- Verify against the built HTML, not the source
- Record the gotcha where docs authors will meet it
