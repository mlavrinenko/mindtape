#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Gate new and changed lines on coverage",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 5,
  links: (
    (
      task: "let-rules-and-templates-choose-where-new-tasks-land.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "gating", "tests", "stateless"),
  x-goals: (
    product: verdicts(
      (enforced-practice: helps[a dead guard fails the gate before review]),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[new code pays its coverage debt instead of freezing it],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 11),
)

== AS-IS

+ Reviewing the on.add commit, where every configured gate passed despite two unreachable branches.
+ 🔴 #raw("Justfile"): The existing gates check absolute size, complexity and score, but no gate checks that new lines are covered.

```
"function": "parse_template_comparison",
"cyclomatic": 7.0,
"coverage": 85.71428571428571,
"crap": 7.142857142857143,
```

`old_value` sits at 80.0%. A fresh `diff-cover --fail-under=90` over this branch reports 96%, 73 missing lines, and exits 0, so an aggregate 90% floor passes the motivating guards. The CRAP gate stops at 30, the undertested arm at 80% with CC 5, and neither fails a new function at 85.7%.


== TO-BE

- 🟢 `just cover` fails when any line added since `main` is uncovered.
- 🟢 A branch commit and a dirty worktree both count.
- 🟢 Existing low-coverage functions stay out of the report.

```
$ just cover
crates/mindtape/src/config/add_hooks_when.rs (95.8%): Missing lines 120,136,139,142
error: changed lines are not covered
```


== Changes

- `nix/dev/shell.nix`: add `python3Packages.diff-cover`, the package's actual nixpkgs attribute.
- `.just/scripts/cover.sh`: after the lcov check, run `diff-cover target/coverage/lcov.info --compare-branch=main --fail-under=100`, with the worktree diff included so uncommitted lines count.
- `docs/src/contributing/coverage-crap.typ`: document the gate and its base branch; the Markdown is generated.


== Out of scope

- Mutation testing, which stays a manual sweep because a full run is hours.
- An aggregate floor below 100%, which passes this branch at 96%, and a run on `main` itself, where the diff is empty.


== Dogfood

This repo's next branch fails on an uncovered new guard before review.

