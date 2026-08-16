#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "exclude generated outputs from the estimate drift check",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("mt", "docs"),
  estimate: (files: 4),
  status: done(2026, 8, 16),
)

== Summary

`[tasks.estimate.drift]` counts every file the working tree touches against
the in-flight task's declared budget. A docs change now touches at least twice
what its author edited: `docs/src/agents.typ` and `docs/src/skill.typ` are two
files, but `just docs md` writes `AGENTS.md` and `skills/mindtape/SKILL.md`
beside them, so the warning fires at four against a truthful estimate of two.

Nobody hand-edits a generated file — `docs::md-check` fails the build if
anyone tries. Counting them turns an estimate into a number an author has to
inflate to keep quiet, which is precisely the failure the `files` unit was
chosen to avoid.

== Scope

The check needs a way to know a path is derived. `.linecop.yaml` already
solves the same problem for size caps, and `docs/generate-md.sh` already knows
every output path (each source declares its own `<mindtape-md>` block). Pick
one:

- a `[tasks.estimate.drift] exclude` list of globs in `.mindtape/config.toml`,
  which is general and costs one config key; or
- teach the check to skip paths matched by an existing ignore mechanism, which
  is narrower and couples MindTape to a convention it does not own.

Prefer the config key: MindTape constrains only what a project declares, and
"which of my files are generated" is exactly a project fact.

== Note

Filed from a delegated agent's observation during the docs de-slop, not from a
sweep. The warning is advisory, so this has never blocked anything — it just
trains everyone to read past it, which is how an advisory signal dies.
