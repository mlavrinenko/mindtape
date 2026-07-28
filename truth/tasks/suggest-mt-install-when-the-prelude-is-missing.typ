#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Suggest mt install when the prelude is missing",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 4),
  status: wip(
    2026,
    7,
    28,
  )[starting: install hint + README/quickstart install story],
)

== Summary

A project whose tasks import \@local/mindtape but whose machine has never run mt install fails silently: every task file errors with a raw Typst diagnostic (eval error: file not found (searched at ~/.local/share/typst/packages/local/mindtape/\<v>/typst.toml)), mt ls prints an empty table, and the exit code is 0. Nothing names the one command that fixes it. This is the first thing a new user or a collaborator cloning a MindTape repo hits, and it reads as the tool being broken rather than unconfigured.

== Scope

- Detect the unresolved \@local/mindtape package among the load errors and print one actionable hint naming mt install, once per run rather than once per file.
- Route every load-error printing site through it (mt ls, mt check, mt flip).
- Keep the raw Typst diagnostic; the hint is an addition, not a replacement.
- Regression test: a project with no installed prelude reports the hint exactly once.
