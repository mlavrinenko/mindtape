#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Require a Docs-Unaffected trailer when an outdatty re-record edits no dependent",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "docs"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the review the drift gate asks for is a line in the commit, refused when missing],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[a re-record either edits a dependent or names the group its author read and left as is],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 17),
  links: parent("keep-the-user-docs-generated-or-gated-not-restated.typ")
    + depends-on(
      "split-the-cli-docs-drift-group-by-the-prose-each-source-feeds.typ",
    )[with cli-docs whole, every command-module commit would need the trailer, so the split makes it name a real review],
)

== Problem

+ Change `crates/mindtape-cli/src/cmd/ls/mod.rs`, run `just outdatty-update cli-docs`, and commit with no doc edit, as `dc34aced` did.
+ 🔴 The commit lands, and its `outdatty.lock` diff moves the source hash. The log can't tell a review that found nothing from a re-record nobody read.
+ 🔴 The same diff moves `www/content/commit-rules.typ`'s hash with no edit to that file, carried along by the group re-record, so the lock alone can't show a review either.


== TO-BE

- 🟢 The commit-msg hook refuses a commit whose staged `outdatty.lock` moves a group's source hash while the commit changes none of that group's dependent files, unless the message names the group:
```text
Docs-Unaffected: cli-ls
```
- 🟢 The refusal names the group and its dependents, so the author reads them before writing the line.
- 🟢 A dependent counts only when the commit changes the file. A hash the re-record carries along doesn't.


== Changes

- `.just/scripts/outdatty-review-trailer.sh`: the red check first: a fixture repo whose commit re-records a group with no dependent edit fails, and passes with the trailer. Then the check, reading each group's sources and dependents from `outdatty.lock`.
- `.githooks/commit-msg`: run it as a fourth check.
- `docs/src/contributing.typ`, `docs/src/contributing/commit-messages.typ`: the trailer, then regenerate their Markdown.


== Rollback

- Drop the call from `.githooks/commit-msg`. The script reads files and writes none.


== Out of scope

- CI. The hook runs where the re-record happens, and a missing trailer on `main` costs a reread, not a broken build.
- A source change that also edits a dependent. That commit already shows its review.


== Dogfood

This repo's next commit that re-records a group without a doc edit carries `Docs-Unaffected:` with the group's name.

