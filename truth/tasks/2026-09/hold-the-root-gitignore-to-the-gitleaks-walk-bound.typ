#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "hold the root gitignore to the gitleaks walk bound",
  priority: ice(
    impact: 3,
    confidence: 0.7,
    ease: 3,
  ),
  difficulty: 3,
  tags: ("gating",),
  estimate: (files: 2),
  assignee: "agent",
  status: done(
    2026,
    9,
    8,
  )[bounded /.direnv, /.mindtape/cache/, .mmz/cache/ and /skills/\*-workspace. /.cargo/\* and /result exempt, with the reason recorded in .gitignore. The arm covers both files, refuses a pattern that hides a tracked file, and has a control pair.],
)

== Summary

Arm four of \`.just/scripts/check-secrets.sh\` holds the \`.gitleaks.toml\` walk
bound to `www/.gitignore`, and stops there. The root `.gitignore` has no such
check, and it has drifted. The bound never lists the gitignored scratch in
`/.direnv`, `/result`, `/.mindtape/cache/`, `/skills/\*-workspace` and `.mmz/cache/`.
I measured it by planting an `aws-access-token` under `.direnv/` and seeing
gitleaks report it. `target/` is the one that matters, for the same reason
`www/generated/` did: `just test`, `just clippy` and `just cover` churn it in
parallel with the scan. The bound covers it, but nothing would notice if that
stopped.


== Scope

- decide, entry by entry, which root-anchored `.gitignore` entries belong in
  the walk bound. `/.cargo/\*` carries a tracked exception
  (`!/.cargo/mutants.toml`) that bounding the directory would hide from the
  scan. `/result` is a nix symlink gitleaks skips anyway. And
  `/skills/\*-workspace` is a glob that path regexes cannot mirror.
- extend arm four to whatever survives that decision, or record the exemptions
  where the arm can read them, so the answer is a check rather than a habit
- keep every pattern in the `(/|\$)` form. `^target/` never matches the
  directory `target` itself. The walk then enters it and stat-s each child.

