#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Count a commit's staged files by glob in a commit rule",
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
      implementable-tasks: neutral,
      enforced-practice: helps[a commit rule can cap how many source files a commit changes, so a tweak stays one file],
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
  status: proposed(2026, 10, 3),
)

== Problem

+ Adding a commit rule that caps a tweak's commit at one source file
+ 🔴 #raw("crates/mindtape/src/commit_expr.rs"): a commit rule cannot count \`files\`, and \`changed()\` is refused there

`assert = 'changed("staged", "!README.md", …) <= 1'` in a `[[commits.rules]]` entry fails config load with `invalid filter: expected a field, got FunctionIdentifier { identifier: "changed" }`. `files` only takes `has`, `miss` and `matches`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/commit_expr.rs").

== TO-BE

- 🟢 A commit rule counts the commit's files a glob list admits, with the same `"!glob"` excludes `changed()` takes, so `.mindtape/config.toml` can cap a tweak's commit at one source file.


== Changes

- `crates/mindtape/src/commit_expr.rs`: evaluate a counting call over `files`.
- `.mindtape/config.toml`: a `tweak-stays-one-file` commit rule.


== Rollback

Revert the commit; a tweak's size is unchecked again.


== Out of scope

- Task-scope `changed()` semantics.


== Dogfood

The `tweak-stays-one-file` rule refuses a tweak commit that grew past one source file.

