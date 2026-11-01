#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Skip merge commits in the commit checks",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("mt", "cli", "gating"),
  estimate: (src-files: 9),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a merge no longer needs a hand-written exemption],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the docs say merges are skipped],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 1),
)

== Problem

+ In a project extending `mindtape:tasks`, make a `--no-ff` merge of a branch that flips a task and changes a product file.
+ Run `mt check --commit-msg .git/MERGE_MSG` while the merge is stopped.
+ 🔴 The merge is judged as one authored commit. `work-names-a-task` and `flip-with-work` refuse it, and `mt check --commits` repeats them on the merge.


== TO-BE

- 🟢 `mt check --commit-msg` exits 0 on a merge and prints `ok: merge commit, commit rules skipped`.
- 🟢 `mt check --commits` skips merge commits, counts them as skipped, and still reads the newest commit's tree for `scope = "tip"` rules.
- 🟢 `--check-merges` checks a merge like any other commit, and `is_merge` is true in its commit expressions.
- 🟢 `git merge --squash` still creates a regular commit, so its message is still checked.

```text
mt check --commit-msg "$1"                    # a merge exits 0
mt check --commits main..HEAD                 # merge commits skipped
mt check --commits --check-merges main..HEAD  # check them
```


== Changes

- `crates/mindtape/src/commit/amend.rs`: detect `MERGE_HEAD` and an amend of a merge, and re-export `writing_a_merge`.
- `crates/mindtape/src/commit/mod.rs`: the `is_merge` record field and the git helpers it needs.
- `crates/mindtape/src/commit/range.rs`: mark a commit with more than one parent.
- `crates/mindtape/src/commit/expr.rs`: read `is_merge` in a commit rule.
- `crates/mindtape-cli/src/cmd/check/commit.rs`: skip a merge before reading the index.
- `crates/mindtape-cli/src/cmd/check/range.rs`: skip merges, keep the newest tree for tip rules, count skips.
- `crates/mindtape-cli/src/cmd/check/mod.rs`: the `--check-merges` flag.
- `crates/mindtape-cli/tests/it/`: the red tests for the hook, the range, the tip, the opt-in and the amend.
- `www/content/commit-rules.typ` and `docs/src/contributing/commit-messages.typ`: say merges are skipped.


== Alternatives

- Check the merge against its first-parent diff by default. It repeats the merged branch, so every rule fires on work the branch commits carry.
- Filter merges in each consumer's hook. Every consumer repeats the guard, and `--commits` still needs an engine flag.


== Out of scope

- Squash merges, which create a regular commit and must still name a task.
- Rewriting a merge message an older `mt` refused. A new merge is enough.


== Dogfood

This repo's `just land --no-ff` and manual merges pass the hook, and CI's `--commits` range no longer fails on one.

