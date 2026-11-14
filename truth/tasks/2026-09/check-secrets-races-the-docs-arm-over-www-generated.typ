#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "check-secrets races the docs arm over www generated",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("gating",),
  estimate: (files: 2),
  assignee: "agent",
  status: done(
    2026,
    9,
    7,
  )[arm four holds the bound to www/.gitignore. A separate task covers the root .gitignore gap.],
)

== Summary

`just check` failed once in three runs with `check-secrets` exiting 8 and no
secret involved. The scan logged `skipping directory: could not get info
error="lstat www/generated: no such file or directory"`, and mmz then
reported `input www/assets/images/.goutputstream-RLQRU3 disappeared after it
was resolved`. Both arms are `just check` running in parallel. The docs arm
rebuilds `www/generated/` and rewrites `www/assets/images/` (via an atomic
save that leaves a `.goutputstream-*` temp) while the secrets arm is walking
the same tree. The next run was green with no change to the worktree.
A gate that fails on timing trains the reader to re-run rather than to read,
which is the property that makes a real leak look like the same flake. The
\`site-pages-fixture-check\` arm already solved the identical race by copying
entry by entry and skipping the directories another arm rewrites. The
secrets walk has no such bound.


== Scope

- reproduce by running \`just check\` against a cold \`www/generated\`, which is
  gitignored and so absent on a fresh checkout. That is the widest window.
- bound the gitleaks walk away from the generated/asset trees another arm
  owns. Or serialise the two arms, or make the walk tolerate a vanishing
  entry without changing what it would report about a tracked file.
- keep the control fixture assertion intact: "no leaks found" must stay
  distinguishable from "loaded no rules"
- the mmz half is the same race and needs the same answer: an input hashed
  while a parallel arm rewrites it can only be advisory

