#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "restore size gate coverage over the dot-just scripts",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (files: 5),
  status: wip(
    2026,
    8,
    15,
  )[linecop 0.4.0 shipped include\_hidden; taking option two],
)

== Summary

linecop skips dot-directories, so every script under `.just/scripts` is
invisible to the size gate the extraction was meant to bring them under.

== Measured, not inferred

A 401-line script placed at `.just/scripts/_probe.sh`:

```
$ linecop            # exactly what check-file-size runs
EXIT=0               # probe not reported at all

$ linecop .just
--- .just/scripts/_probe.sh: 401 lines (limit: 300, +101 over)
EXIT=1
```

The cap works when the hidden directory is the explicit scan root, and only
then. Nothing is wrong with the `Shell: 300` limit; the walker never reaches
the files.

== Why this matters more than it looks

The whole stated reason for moving recipe bodies out of the Justfile was that
`Justfile` has no extension, so linecop cannot see it, and a `.sh` file lands
under a real cap on arrival. That reason is currently void: the bodies moved
from one blind spot into another. The Justfile shrank from 949 lines to 763,
which is real, but no size gate is holding the scripts to anything.

This is the same class of failure as the one this repo already documented about
`.githooks/`: linecop detects by extension and those hooks are shebang-only, so
they are invisible at any limit. Here the mechanism is different — the directory
is hidden, not the extension missing — but the outcome is identical and it was
introduced, not inherited.

== Three ways out, in preference order

+ Give `check-file-size` a second scan root. `linecop && linecop .just` stays a
  single line, so the gate table's Command column keeps rendering the real
  command rather than falling back to the doc string. Cheapest, in-repo, no
  upstream dependency.
+ Teach linecop to scan hidden directories. It is this author's own tool, and
  its config schema has no include-hidden key today — `limits`, `overrides`,
  `exclude_dirs`, `count_mode` only. This is the fix that stops the next project
  hitting it, and it makes the first option unnecessary.
+ Drop the leading dot: `just/scripts/`. Cleanest for tooling, worst for the
  goal that motivated `.just/` — keeping build-tool clutter out of the repo
  root.

Do not solve it by turning `check-file-size` into a multi-line body without
thinking: that changes what `www/gates.jq` renders into the gate table's Command
column, which is a rendered doc diff.

== Scope

- Pick one and say why in the commit.
- Prove it bites: an oversized file under `.just/scripts` must fail
  `check-file-size`. Assert the probe file actually exists before asserting the
  gate fails — this repo has already had a negative test pass against a file
  that was never written.
- If the fix is upstream in linecop, file the issue there and take option one in
  the meantime rather than leaving the gap open.
