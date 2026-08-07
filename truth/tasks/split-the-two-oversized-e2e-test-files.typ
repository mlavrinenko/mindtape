#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "split the two oversized e2e test files",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tests",),
  estimate: (files: 4),
  status: proposed(2026, 8, 8),
)

== Summary

\`crates/mindtape-cli/tests/add\_e2e.rs\` and \`tests/init\_e2e.rs\` both sit at 490 of the 500-line cap. Integration test files in \`tests/\` are standalone binaries, so nothing mechanical splits them; the next e2e case added to either hits the wall.

== Scope

- Split each along a subject seam into a second integration binary under \`crates/mindtape-cli/tests/\`.
- Shared helpers already live in \`tests/support/mod.rs\` — reuse it rather than duplicating setup into the new file, which would also trip the jscpd gate.
- No behaviour change.
