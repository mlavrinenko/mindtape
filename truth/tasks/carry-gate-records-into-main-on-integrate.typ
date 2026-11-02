#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Carry gate records into main on integrate",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "tooling"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[the gate the hook asserts stays fresh across integrate, so no one is tempted to bypass it],
      ),
      "readable-tasks",
      "implementable-tasks",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (
        sound-codebase: helps[integrate stops discarding a verified gate result],
      ),
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 7),
)

== AS-IS

+ Committing after a just integrate
+ 🔴 #raw(".just/scripts/integrate.sh"): integrate.sh asserts the gate in the worktree (line 157), then fast-forwards (line 170) without telling main's .mmz/cache. Main's gate is stale, so .githooks/pre-commit refuses the next commit until just check re-runs: about 10 minutes, twice today.

mmz records are portable: `.mmz/cache/SLUG-HASH.yaml` holds command, input_digest, status and ran_at, and the digest comes from repo-relative paths and content hashes. Not tested across two live checkouts.

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("a copy step in .just/scripts/integrate.sh").

== TO-BE

- 🟢 After integrate, `mmz --is-fresh --tag gate` in main exits 0 on the merged tree, and the next commit is not refused.

== Changes

- `.just/scripts/integrate.sh`: after line 170, copy the worktree's `.mmz/cache/*.yaml` into main's; mmz stays the judge, so a record whose digest differs stays stale.
- Red test first: integrate a fixture branch, assert `mmz --is-fresh --tag gate` in main exits 0.

== Rollback

- Revert the commit; main's gate goes stale after integrate again.

== Out of scope

- Running the gate in main, or changing which scopes exclude the task files.

== Dogfood

The next integrate in this repo, then a task-only commit, exercises it.
