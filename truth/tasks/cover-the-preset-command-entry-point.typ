#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "cover the preset command entry point",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tests",),
  estimate: (files: 1),
  status: proposed(2026, 8, 1),
)

== Summary

\`preset\_cmd::run\` is the only function above the CRAP threshold on a fresh lcov: score 42, cyclomatic complexity 6, 0% covered. \`crates/mindtape-cli/tests/\` has no \`preset\_e2e.rs\` — the unit tests in \`preset\_cmd\_tests.rs\` call \`save()\` and \`render\_ls()\` directly, so the command entry point that resolves the root and the user config dir is never executed. \`preset\_cmd.rs\` has one commit (1238df1) and no preset e2e has ever existed on any branch, so the function has been uncovered since it landed. \`just crap\` fails on a fresh lcov for this one function alone (verified locally); the CI \`coverage\` job runs \`just cover\` then \`just crap\` in the same job with no \`continue-on-error\`, so that step fails wherever it runs — not verified against actual run history.

== Scope

- add \`crates/mindtape-cli/tests/preset\_e2e.rs\` driving \`mt preset save\` and \`mt preset ls\` as subprocesses; \`support::mt\` already isolates \`MINDTAPE\_CONFIG\_HOME\`/\`XDG\_CONFIG\_HOME\`, which is what makes the user-preset dir testable
- llvm-cov merges child profraw, so a subprocess e2e credits \`run\` directly
- out of scope: lowering the threshold or \`--allow\`-ing the function — the gap is real, not a metric artifact
