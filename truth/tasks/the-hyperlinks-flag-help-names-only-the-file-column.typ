#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "the hyperlinks flag help names only the file column",
  priority: ice(
    impact: 2,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("docs",),
  estimate: (files: 2),
  assignee: "agent",
  status: proposed(2026, 9, 8),
)

== Summary

`mt ls --help` describes `--hyperlinks` as "Hyperlink output for the `file`
column", while `crates/mindtape/src/config_ls.rs:69` and the skill both say
the flag wraps the `#` and `file` columns. The flag doc is the stale one:
`#` is always visible, so it is the cell a reader is most likely to click,
and the help omits it.


== Scope

Fix the `--hyperlinks` doc comment on the CLI arg so it names both the `#`
and `file` columns, matching `crates/mindtape/src/config_ls.rs:69`. Check
whether the captured CLI output under `www/generated/` needs regenerating
with `just docs-gen`, since the docs site reads the help text rather than
restating it.

