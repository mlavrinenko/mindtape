#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "split the near-cap unit test modules",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tests",),
  estimate: (files: 11),
  status: done(
    2026,
    8,
    8,
  )[five modules split by named theme; parents grew 4 lines each, recur.rs now 481/500],
)

== Summary

Five ejected test modules sit at 91-96% of the 500-line cap: \`add\_interactive\_tests.rs\` (482), \`add\_values\_tests.rs\` (475), \`ls\_rules\_tests.rs\` (478), \`check\_tests.rs\` (461), \`recur\_tests.rs\` (458, the mindtape-cli one). They are already ejected — each parent carries \`\#\[path = "...\_tests.rs"\] mod tests\` — so \`just eject\` has no second move and the next test added to any of them hits the wall mid-edit.

== Scope

- Split each by theme into a sibling \`\*\_tests\_\<theme>.rs\`, wired with a second \`\#\[path\]\` module in the parent.
- Do not rebalance by line count alone: the seam must be a subject a reader can name.
- \`crates/mindtape-cli/src/recur.rs\` is itself at 477 lines; adding the second \`\#\[path\]\` line there must not push it over. If it would, that file needs its own split first — say so rather than trimming prose to fit.
- No behaviour change: the same tests, same names, same assertions.
