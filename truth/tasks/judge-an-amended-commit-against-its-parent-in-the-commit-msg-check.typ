#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Judge an amended commit against its parent in the commit-msg check",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[closing a task by amending its work commit passes flip-with-work],
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
  status: proposed(2026, 10, 7),
)

== Problem

+ Running \`mt done X\` then \`git commit --amend --no-edit\` on the commit holding X's work.
+ 🔴 #raw("crates/mindtape/src/commit_msg.rs"): \`staged\_files\` diffs the index against HEAD (\`--cached\`), so during an amend \`files\` holds only the flip, \`tasks\_only\` is true, and rule \`flip-with-work\` refuses a commit that does carry the work.

```
start or close a task in a commit that carries its work
```
`crates/mindtape/src/commit_msg.rs:188`: `paths: changed_paths(root, &["--cached"])?,`
`crates/mindtape/src/commit_msg.rs:189`: `renamed_from: renamed_sources(root, &["--cached"])?,`
`crates/mindtape/src/packs/mindtape-tasks.toml:360`: the rule asserts `!tasks_only`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/commit_msg.rs").

== TO-BE

- 🟢 During `git commit --amend`, `files` lists the paths the amended commit changes against `HEAD^`, so a flip added to a work commit passes `flip-with-work`.

== Changes

- `crates/mindtape/src/commit_msg.rs`: detect an amend in `staged_files` and pass `HEAD^` as the diff base in place of HEAD. Add a red test with an amended commit first.

== Rollback

- Revert the `commit_msg.rs` change.

== Out of scope

- `mt check --commits <range>`, which already reads whole commits.

== Dogfood

`mt done` plus `git commit --amend --no-edit` becomes a valid way to land the flip in this repo.
