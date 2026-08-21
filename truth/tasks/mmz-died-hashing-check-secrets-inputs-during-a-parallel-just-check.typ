#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mmz died hashing check-secrets inputs during a parallel just check",
  priority: ice(
    impact: 4,
    confidence: 0.5,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("gating",),
  estimate: (files: 1),
  assignee: "agent",
  status: proposed(2026, 8, 21),
  links: related(
    "the-site-pages-replica-copies-www-public-from-a-parallel-arm.typ",
  )[
    found in the `just check` run that closed that one — the same family, one
    layer out: not a gate reading a directory a parallel arm owns, but a
    gate's memoizer doing it
  ],
)

== Summary

`just check` failed once with `mmz: i/o error: No such file or directory (os
error 2)` on the `check-secrets` arm — exit 70, after gitleaks itself had
already printed `no leaks found`. So the gate PASSED and its memoizing wrapper
is what died, turning a green arm into a red aggregate. The next full run and a
direct `just check-secrets` both came back clean, and nothing in the log says
which file went missing.

`check-secrets`' `secrets-tree` scope is the widest in .mmz/config.yaml — `*`
plus `www/**`, `crates/**`, `.just/**` and six more — and `just check` runs
around twenty arms in parallel, several of which rewrite files while it hashes.
mmz does NOT hash gitignored files (measured: touching `www/generated/ZPROBE`
leaves the rule fresh), so the churn from `www/generate.sh`'s `rm -rf` is ruled
out and whatever vanished was TRACKED, or belonged to mmz's own cache
bookkeeping under concurrent writers. Both are guesses.

== Scope

- Establish which file mmz was reading. `-v`/`-vv` on the wrapper, or a run
  under `strace -f -e trace=file`, should name it; a stale-input hash race and
  a cache-write race want different fixes
- If it is a tracked file, find the arm that rewrites it mid-run and either
  narrow the scope or serialize the two, the way the www/generated flock
  already serializes its writers
- If it is mmz's own cache under concurrent writers, it is an upstream bug —
  report it there rather than working around it here
- Do not paper over it by dropping `-e` or swallowing exit 70: an arm whose
  wrapper can fail independently of the gate is exactly the flake the
  site-pages family of tasks has been closing
- Filed unreproduced, on purpose. One occurrence in one log is thin, but a red
  `just check` that names a gate the change never touched is the shape this
  repo has now hit three times, and the second occurrence should land on an
  existing task rather than start the investigation over
