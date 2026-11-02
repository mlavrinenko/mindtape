#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Record the dev-shell-docs outdatty group after the wt.toml comment change",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "tooling"),
  x-goals: (
    product: verdicts(
      (:),
      "readable-tasks",
      "implementable-tasks",
      "enforced-practice",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (
        sound-codebase: helps[main passes just check again, so every just land can fast-forward],
      ),
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
)

== AS-IS

+ Running just land from a worktree rebased onto main at 06a0492f
+ 🔴 #raw("outdatty.lock"): 06a0492f edited .config/wt.toml, the source of the dev-shell-docs group, without recording the group, so outdatty-check fails on main and blocks every just land

```
[ stale ]  dev-shell-docs
    source changed:    .config/wt.toml
    confirm with:      outdatty update --group dev-shell-docs
outdatty: 1 of 22 groups out of date
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("outdatty.lock").

== TO-BE

- 🟢 `just outdatty-check` passes on main after the group is reviewed and recorded.

== Changes

- `outdatty.lock`: record the dev-shell-docs group after reviewing its dependents against the new comment.

== Rollback

- Revert the commit.

== Out of scope

- Why the commit passed its gate. If the gate was skipped, that is its own finding.

== Dogfood

Every just land works again.
