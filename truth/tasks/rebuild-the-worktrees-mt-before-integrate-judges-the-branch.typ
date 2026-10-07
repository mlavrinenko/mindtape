#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rebuild the worktree's mt before integrate judges the branch",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[integrate judges a rebased branch by its own config, not a stale binary],
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
  status: done(2026, 10, 7),
)

== Problem

+ Integrating a branch rebased over a config change.
+ 🔴 #raw(".just/scripts/integrate.sh"): \`integrate.sh\` runs \`mt\` in the worktree, but that binary was built once by \`fork.sh\`; after a rebase over a config change it rejects the new config, so a good branch is refused.

```
invalid config: commits.rules[2].when: unknown field in filter: trailers
```
`.just/scripts/integrate.sh:120`: `key=$(run_in_worktree mt config show --json | jq -r '.commits.trailer // "Refs"')`
`.just/scripts/integrate.sh:123`: `open=$(run_in_worktree mt ls 'status == "wip"' --format path \`
`.just/scripts/fork.sh:142-143`: builds the worktree's `mt` once with `just build-mt`.
Running `just build-mt` in the worktree cleared the refusal.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/integrate.sh").

== TO-BE

- 🟢 `just integrate` rebuilds the worktree's `mt` from the branch tip before its first `mt` call, so a rebased branch is judged by its own config.

== Changes

- `.just/scripts/integrate.sh`: run `run_in_worktree just build-mt` before the `mt config show` call, skipping it when the recipe is absent as `fork.sh` does.

== Rollback

- Revert the `integrate.sh` line.

== Out of scope

- Running the gates. Integrate still only asserts a prior pass.

== Dogfood

Every `just integrate` of a rebased branch here.
