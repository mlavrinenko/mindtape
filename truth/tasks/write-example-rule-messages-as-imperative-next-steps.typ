#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write example rule messages as imperative next steps",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 1,
  estimate: (src-files: 4),
  tags: ("found", "cli"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[every message follows the error format],
      ),
      "readable-tasks",
      "implementable-tasks",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (:),
      "sound-codebase",
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 8),
)

== AS-IS

+ Reading config errors against the error format in diag/mod.rs
+ 🔴 #raw("examples/rules/.mindtape/config.toml"): example rule messages read as statements, not imperative next steps like the shipped pack's

message = "more than one em-dash in a paragraph reads as generated prose"


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("examples/rules/.mindtape/config.toml").

== TO-BE

- 🟢 Every example rule message tells the reader what to do.

== Changes

- `examples/rules/.mindtape/config.toml`: reword each message; regenerate the docs.

== Rollback

- Revert the commit.

== Out of scope

- The cmd/ modules other tasks convert.

== Dogfood

The docs pages quote the reworded messages.
