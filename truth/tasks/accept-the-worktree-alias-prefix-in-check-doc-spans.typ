#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Accept the worktree alias prefix in check-doc-spans",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
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
      sound-codebase: neutral,
      docs-match-code: helps[a doc can name every form mt accepts],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 3),
)

== Problem

+ Running just check-doc-spans over the trimmed mindtape skill.
+ 🔴 #raw(".just/scripts/check-doc-spans.jq"): check-doc-spans reads the word after mt as the subcommand, so it refuses \`mt \@name ls\`, a form mt accepts.

For the span `mt @name ls` in `docs/src/skill.typ:38`, `just check-doc-spans` printed `no subcommand @name; closest: add`. `crates/mindtape-cli/src/worktrees.rs` rewrites a leading `@name` to `-w <path>` before clap parses, so `mt @sn ls` runs. The skill now writes `@name` in a span of its own to pass.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/check-doc-spans.jq").

== TO-BE

- 🟢 `check-doc-spans` skips a leading `@name` after `mt` and checks the subcommand that follows it.
- 🟢 The gate's control holds an `mt @name ls` span among the good shapes.


== Changes

- `.just/scripts/check-doc-spans.jq`: drop a leading `@word` before reading the subcommand.
- `.just/scripts/check-doc-spans.sh`: add the good shape to the control page.


== Rollback

Revert the commit.

== Out of scope

- Checking that the alias exists: aliases live in each user's `worktrees.toml`.


== Dogfood

The mindtape skill can show the alias form as a command again.
