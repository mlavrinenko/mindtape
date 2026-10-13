#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Hash every outdatty path in the outdatty-check rule",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a gate stays red when an outdatty source changes outside its inputs],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
)

== Problem

+ Landing 06a0492f, which edited .config/wt.toml and went red on main
+ 🔴 #raw(".mmz/config.yaml"): the `just outdatty-check` rule takes no scope holding .config/wt.toml, .gitattributes, the .githooks files or release.toml, so an edit to those outdatty sources serves a false-fresh gate

`outdatty: dev-shell-docs, source changed: .config/wt.toml` passed the land gate as fresh

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/script-scope-check.sh").

== TO-BE

- 🟢 Every outdatty.yaml source and dependent is hashed by the outdatty-check rule, and `just script-scope-check` fails by name on one that is not.

== Changes

- `.mmz/config.yaml`: an `outdatty-sources` scope for the uncovered paths. `.just/scripts/script-scope-check.sh`: arm four.

== Rollback

- Revert the commit.

== Out of scope

- The existing three arms.

== Dogfood

Dropping the scope makes `just script-scope-check` name .config/wt.toml.
