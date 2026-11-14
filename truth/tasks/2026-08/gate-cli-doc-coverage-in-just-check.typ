#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "gate cli doc coverage in just check",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("gating", "docs"),
  estimate: (files: 6),
  status: done(
    2026,
    8,
    8,
  )[check-doc-coverage wired as a just check arm, exposed and closed a 5-command pre-existing gap (graph, import, init, preset, worktrees)],
)

== Summary

`mt find` landed with no entry in `www/utils/cli-notes.typ`. The docs site would have carried a new verb with no prose or examples while every sibling command has them. The outdatty `cli-docs` group knew `cli.rs` had changed but not that a command lacked docs. A human caught it during a dependents review, which is not a mechanism. The check is exact and cheap: `mt _cli --json` reflects the whole command tree, and `cli-notes.typ` is a keyed dict.

== Scope

- Add a `check-doc-coverage` recipe: set-difference the visible subcommand names from `mt _cli --json` against the keys of `www/utils/cli-notes.typ`, fail naming each undocumented command.
- Exclude hidden and internal commands (`_cli` itself, the bare status sugar) by what the dump reports, not by a hand-maintained skip list that would rot the same way.
- Wire it as a `just check` arm and add the matching `.mmz/config.yaml` rule with `tags: [gate]`, inputs `rust` + `docs` + `gates-meta`.
- The Justfile is an outdatty `dev-docs` source, so reconcile CONTRIBUTING.md and AGENTS.md and re-record `outdatty.lock`. CONTRIBUTING.md is at 200 of its 200-line cap. Compact it deliberately or raise the limit with a written reason, and do not pay for the new gate in reworded prose.
