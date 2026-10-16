#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add an in_range mode predicate to commit rules",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.6,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("decision", "gating"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a rule refuses an unsquashed fixup in a range, so a hook script is not the only guard],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 16),
)

== Problem

+ Run `mt check --commit-msg msg` with a `fixup! feat: x` subject, then `mt check --commits main..HEAD` over a branch holding that commit.
+ 🔴 No commit rule can pass the first and refuse the second: both modes run the same commit-scope rules over the same fields, and a tip rule never sees a commit that has no `Refs:`.

== TO-BE

- 🟢 A commit-scope rule reads `in_range`, true under `mt check --commits` and false under `mt check --commit-msg`.

```toml
[[commits.rules]]
name = "no-unsquashed-fixup"
when = 'in_range && matches(subject, "^(fixup|squash|amend)! ")'
assert = "false"
message = "run git rebase -i --autosquash"
```

== Changes

- `crates/mindtape/src/commit/expr.rs`: the red test first, then read `in_range` as a bool field.
- `crates/mindtape/src/config/commits.rs`: pass the mode into `check_commit`.
- `www/content/commit-rules.typ`: document the field.
- `.config/wt.toml`: drop the fixup hook once the rule replaces it.

== Alternatives

- A range scope (`scope = "range"`) over every commit's subject with an aggregate: the cleanest model, but a new scope and evaluator.
- Expose the range's subjects to a tip rule: a bigger change than the predicate.
- Keep the `pre-merge` hook script: no language change, but a rule in `mt rules` cannot show it.

== Out of scope

- The `work-names-a-task` guard, which already exempts a fixup in both modes.
- Owner call: the predicate breaks "the same rules run in both modes", so this task records the decision before any code.

== Dogfood

This repo's `.config/wt.toml` fixup hook becomes one rule in `.mindtape/config.toml`.
