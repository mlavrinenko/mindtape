#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "ls limit flag",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 2),
  assignee: "max",
  status: done(
    2026,
    9,
    3,
  )[the -n/--limit flag already shipped; what landed here is the truncation signal — a truncated: table footer, a truncated key in the --json envelope, nothing in csv/path/path0],
)

== Summary

\`mt ls\` and \`mt find\` can only be truncated by piping into \`head\`, which is
exactly the invocation that trips the intermittent SIGPIPE exit 141. A native
limit flag gives callers a truncation path that does not depend on the pipe
behaving, and is useful on its own for agents that want a bounded listing.


== Scope

- Add \`-n \<N>\` / \`--limit \<N>\` to \`mt ls\` (and therefore \`mt find\`, which forwards every \`ls\` flag).
- Truncate after filtering and sorting, so the flag composes with \`--sort\` and named views.
- Apply to every \`--format\` (text, json, csv, path, path0), not just the human table.
- Decide whether a truncated listing should signal that more rows exist; a footer line in text format and nothing in machine formats keeps \`--json\` parseable.
- Complementary to, not a substitute for, the SIGPIPE fix: other pipe targets still break.

