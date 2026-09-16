#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Assert a branch's gate before fast-forwarding it",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling", "gating"),
  estimate: (files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a merge asserts the branch's own gate instead of trusting the integrator's shell pipeline],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a red branch cannot land on `main` unnoticed],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    9,
    16,
  )[integrate recipe plus .just/scripts/integrate.sh, with the wiring three gates demand: a script-integrate mmz scope, an outdatty dev-docs source, and the regenerated CONTRIBUTING.md — 7 files against the 3 estimated. All five paths verified by hand against scratch worktrees: no worktree for the branch, a dirty worktree, a non-fast-forward, a cold gate naming all 23 arms, and the green path — main's .mmz/cache transplanted into a probe at HEAD, where 22 arms read fresh and check-crap correctly reported stale on the absent target/coverage/lcov.info until that artifact was transplanted too, after which the host worktree fast-forwarded 47d0534 to 0865fff and the probe was removed.],
)

== Problem

+ Run `just fork <slug>`, let an agent commit on the branch, then read its gate with a pipeline ending in `tail`.
+ 🔴 The pipeline reports `tail`'s exit code, so a red gate reads as green. This happened here: a branch whose test arm failed was fast-forwarded into `main`.
+ Run `git merge --ff-only <branch>` on `main`.
+ 🔴 Nothing asserts the branch ever passed. A fast-forward writes no commit, so `main`'s pre-commit hook never runs, and the only check is whatever the integrator typed.


== TO-BE

- 🟢 `just integrate <branch>` refuses unless that branch's worktree reports `mmz --is-fresh --tag gate`, the same assertion the pre-commit hook makes, then fast-forwards and removes the worktree.
- 🟢 A refusal names the stale or failed arm, and says the fix is a fresh `just check` in that worktree.
- 🟢 `just fork` and `just integrate` read as the pair they are, one standing a worktree up and one taking it down.

```text
$ just integrate check-fn
integrate: ../mindtape-check-fn: gate is stale — re-run `just check` there
$ just integrate body-rule-names
integrate: ../mindtape-body-rule-names: gate fresh; main fast-forwarded to 20e1f6a, worktree removed
```


== Changes

- `Justfile` (or `.just/`): the `integrate` recipe beside `fork`, with its body in `.just/scripts/integrate.sh` if it outgrows the cap `fork` respects.
- `docs/src/contributing.typ`: name it beside `just fork`, then `just docs md`.
- `outdatty.lock`: re-record the group the Justfile belongs to.


== Rollback

Revert the commit: `git merge --ff-only` by hand is what exists today, and nothing outside the repo changed.


== Out of scope

- Running the gate: the recipe asserts freshness and never re-runs it, the same trade the pre-commit hook takes.
- Rebasing a branch onto `main`: conflicts stay a person's job.


== Dogfood

Built in: every delegated worktree in this repo would merge through it.

