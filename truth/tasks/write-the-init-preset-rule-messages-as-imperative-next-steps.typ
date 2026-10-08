#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write the init preset rule messages as imperative next steps",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[every message follows the error format],
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
  status: proposed(2026, 10, 8),
)

== Problem

+ Rewording the example rule messages
+ 🔴 #raw("crates/mindtape-cli/src/cmd/init/presets/gated/config.toml"): the init presets word rule messages as statements, not imperative next steps

message = "a task needs exactly one Summary: the problem, stated once"


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/init/presets/gated/config.toml").

== TO-BE

- 🟢 Every init preset rule message tells the reader what to do.

== Changes

- `crates/mindtape-cli/src/cmd/init/presets/gated/config.toml` and `axes/headers_spec.toml`: reword each message.

== Rollback

- Revert the commit.

== Out of scope

- The Rust in cmd/init.

== Dogfood

A gated init shows the reworded messages.
