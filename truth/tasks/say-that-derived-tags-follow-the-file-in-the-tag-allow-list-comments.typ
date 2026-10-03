#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Say that derived tags follow the file in the tag allow-list comments",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
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
      docs-match-code: helps[the comment and the schema text describe what the loader does],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== Problem

+ Rewriting /derived-tags/, which made the same claim
+ 🔴 #raw("crates/mindtape/src/validate.rs"): The check\_tags comment says a closed task keeps the tags its folder derived when it was filed, but the loader derives folder tags again on every load.

`crates/mindtape/src/validate.rs:368` reads "a closed task keeps the tags it was filed with, authored and derived alike", while `crates/mindtape/src/loader.rs:178` sets `task.derived_tags = item.folder_tags.clone()` on every load. The `[tasks.tags] allowed` description in `crates/mindtape/src/config_tasks.rs:207` (and `schema/config.schema.json`) says "a terminal task keeps the tags it was filed with", which reads the same way.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/validate.rs").

== TO-BE

- 🟢 The comment and the schema description say that `mt check` skips the allow-list for closed tasks, and that a moved file gets new folder tags.


== Changes

- `crates/mindtape/src/validate.rs`: fix the `check_tags` comment.
- `crates/mindtape/src/config_tasks.rs`: fix the `allowed` description, then regenerate the schema.


== Rollback

Revert the commit.


== Out of scope

- Changing the allow-list behaviour itself.


== Dogfood

Not applicable: comment and description text only.

