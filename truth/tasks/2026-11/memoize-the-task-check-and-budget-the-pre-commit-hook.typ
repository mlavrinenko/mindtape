#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Memoize the task check and budget the pre-commit hook",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "gating", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a commit after just check reuses the task arms instead of re-running them],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 15)[landed: hook 1.6s after a full just check],
)

== AS-IS

+ Do Rust work on `main`, then run `just check`. The gate records pass.
+ 🔴 The next commit still takes 37s. The hook rebuilds `target/mt-dev` from 13:34:09 to 13:34:34, then runs `mt check` for 4.7s:

```
$ time .githooks/pre-commit
real 0m9.6s
```


== TO-BE

- 🟢 `just check` runs `mt check` as the untagged `mt-check` arm. The hook finds that record and takes 1.6s.
- 🟢 A green hook over its 10s budget fails and prints each step's time, naming the step to make cheap.

```
$ time .githooks/pre-commit
real 0m1.6s
```


== Changes

- `.githooks/pre-commit`: assert the task arms through mmz, and fail a green run over the 10s budget with a per-step breakdown.
- `Justfile`: add the `mt-check` gate and put it in `_check-build`.
- `.mmz/config.yaml`: declare the untagged `mt-check` rule and the `truth-shared` scope.
- `.just/scripts/check.sh`: build `target/mt-dev` after the build stage for the commit-msg hook.
- `docs/src/contributing/gates.typ`: document the two untagged arms, the hook budget, and the `mt-dev` build.
- `docs/contributing/gates.md`: the regenerated page.
- `.config/outdatty.lock`: re-record the three groups.


== Out of scope

- The 4.7s `mt check` load itself. It runs only when a task or config changes, and `mt check` has no path filter.
- The commit-msg hook's 1.8s Vale run. The budget guards the pre-commit hook.
- Raising the budget to cover a slow step.


== Dogfood

This repo's commits after `just check` take 1.6s, and a cold `target/mt-dev` build fails the budget instead of hiding in the hook.

