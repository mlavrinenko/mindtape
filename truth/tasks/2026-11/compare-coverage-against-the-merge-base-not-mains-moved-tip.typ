#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Compare coverage against the merge base, not main's moved tip",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  links: (
    (
      task: "show-the-goal-dsl-as-a-project-owned-example.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "gating", "coverage", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 24),
)

== AS-IS

+ Running `just check` in a delegated worktree cut before main advanced
+ 🔴 #raw(".just/scripts/cover.sh"): `just cover` diffs against `main`'s tip, so a branch cut before main's later commits reports their lines as new and uncovered.

```
==> just cover failed (log: target/check/cover.log)
Diff: origin/main...HEAD, staged and unstaged changes
-------------
crates/mindtape/src/commit/state_cache.rs (88.9%): Missing lines 51
crates/mindtape/src/commit/state_loader.rs (85.4%): Missing lines 288-292,307
-------------
cover: new or changed lines must be covered; add a test for the lines named above
```


== TO-BE

- 🟢 `just cover` diffs from the merge base of `main` and HEAD, so it reports only the branch's own lines.
- 🟢 A branch stays green after main lands other work.

```
$ just cover
check: 44 gates passed
```


== Changes

- `.just/scripts/cover.sh`: resolve `base` with `git merge-base "$base" HEAD` before `git diff`.
- `docs/src/contributing/coverage-crap.typ`: say the gate compares against the merge base.


== Out of scope

- The coverage thresholds and the `diff-cover` flags. The base is the defect.


== Dogfood

Every delegated worktree keeps a green `just check` after main moves.


== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a stale base fails a clean branch on lines main landed].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).

