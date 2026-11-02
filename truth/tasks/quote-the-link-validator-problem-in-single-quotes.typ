#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Quote the link validator problem in single quotes",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
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
+ 🔴 #raw("crates/mindtape/src/config/links_validate.rs"): Validators::problem quotes the kind and target in backticks, not the format's single quotes

`{kind}` edge target `{target}` is not a valid `{}`


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/config/links_validate.rs").

== TO-BE

- 🟢 The message quotes with single quotes.

== Changes

- `crates/mindtape/src/config/links_validate.rs`: swap the backticks for single quotes.

== Rollback

- Revert the commit.

== Out of scope

- The cmd/ modules other tasks convert.

== Dogfood

A bad link target reads in the error format.
