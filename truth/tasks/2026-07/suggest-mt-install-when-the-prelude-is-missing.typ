#import "@local/mindtape:0.3.0": *

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
  status: done(2026, 7, 28)[one note per run, wired through warn\_errors],
)

== Summary

Tasks that import \@local/mindtape fail silently on a machine that has never run mt install. Every task file errors with a raw Typst diagnostic (eval error: file not found (searched at ~/.local/share/typst/packages/local/mindtape/\<v>/typst.toml)). mt ls prints an empty table, and the exit code is 0. No message points at the one command that fixes it. A new user, or a collaborator cloning a project built on this tool, meets this first, and it makes the tool look broken rather than unconfigured.

== Scope

- Detect the unresolved \@local/mindtape package among the load errors and print one concrete hint naming mt install, once per run rather than once per file.
- Route every load-error printing site through it (mt ls, mt check, mt flip).
- Keep the raw Typst diagnostic. The hint is an addition, not a replacement.
- Regression test: a project with no installed prelude reports the hint exactly once.
