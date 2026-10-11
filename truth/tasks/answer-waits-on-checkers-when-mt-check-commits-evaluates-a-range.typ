#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Answer waits-on checkers when mt check --commits evaluates a range",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a range check refuses starting a task its checker still holds],
      enforced-practice: helps[`just land` applies the same blocked rule a commit hook does],
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
  status: proposed(2026, 10, 11),
)

== Problem

+ Asked how commit rules see a task that waits on a \`\[\[tasks.waits\]\]\` checker
+ 🔴 #raw("crates/mindtape/src/commit/tasks.rs"): \`ProjectState::new\` builds each commit's tasks without asking their checkers, so in \`mt check --commits\` a waiting task reads ready, while \`--commit-msg\` on the staged tree reads it blocked

```
# rule: when = 'into("wip")', assert = 'staged(!blocked)'
$ mt check --commit-msg msg          # staged: verify-webhooks… waits
warning: msg: commits.rules.start-only-ready-work: verify-webhooks-with-the-acme-pay-sdk: don't start a task that still waits
$ mt check --commits HEAD~1..HEAD    # same flip, committed: silent
$ mt check --commits HEAD~2..HEAD    # a depends-on blocker: fires
warning: f5f2a53: commits.rules.start-only-ready-work: throttle-failed-logins-per-ip: don't start a task that still waits
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/commit/tasks.rs").

== TO-BE

- 🟢 `mt check --commits` reads a waiting task as `blocked` the way `--commit-msg` does, asking each checker once per run with today's answer.


== Changes

- `crates/mindtape/src/commit/tasks.rs`: call `crate::waits::annotate` in `ProjectState::new` with the repository root.
- `crates/mindtape-cli/tests/it/waits_e2e.rs`: the range case above, red first.


== Rollback

- Revert the `annotate` call; commit rules go back to ignoring waits in a range.


== Out of scope

- Answering a checker as of a past commit. A checker reads local state now, and history holds no copy of it.


== Dogfood

Not applicable until a task in this repo uses `waits-on`; `just land` runs `mt check --commits main..HEAD` and would then honour it.
