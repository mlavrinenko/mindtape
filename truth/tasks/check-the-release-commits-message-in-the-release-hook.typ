#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check the release commit's message in the release hook",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a dry run catches every refusal a real release can hit],
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
  status: proposed(2026, 10, 28),
)

== Problem

+ Running `just release 0.3.0`, whose dry run had passed
+ 🔴 #raw(".just/scripts/release-hook.sh"): the release hook never runs the commit rules on the release commit message, so a dry run passes a release whose commit the commit-msg hook then refuses

```
error: .git/COMMIT_EDITMSG: commits.rules.work-names-a-task: name the task this commit works on in a `Refs: <stem>` footer
error: 1 commit message problem
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/release-hook.sh").

== TO-BE

- 🟢 `just release X.Y.Z --dry-run` refuses a release whose `chore(release): X.Y.Z` commit the commit-msg hook would refuse.

== Changes

- `.just/scripts/release-hook.sh`: write the subject from `.config/release.toml` to a temp file and run `mt check --commit-msg` on it, refusing on failure.

== Rollback

- Revert the hook lines.

== Out of scope

- The `work-names-a-task` override in `.mindtape/config.toml`, which already exempts the release subject.

== Dogfood

Every `just release` in this repo checks its own commit message before it pushes.
