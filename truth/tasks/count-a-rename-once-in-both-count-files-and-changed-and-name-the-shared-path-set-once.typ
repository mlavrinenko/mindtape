#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Count a rename once in both count(files) and changed(), and name the shared path set once",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 4,
  estimate: (src-files: 4),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the source-file budget and the tweak cap count the same change the same way],
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
  status: proposed(2026, 10, 4),
)

== Problem

+ Reviewing commit 2a3e070f with a \`git mv\` in a scratch repository
+ 🔴 #raw("crates/mindtape/src/commit_msg.rs"): count(files) counts a rename as two files where changed() counts one, and the 14-glob exclude list is copied into both src-budget and tweak-stays-one-file.

After `git mv src/old.rs src/new.rs`, the task rule `changed("staged", "src/**") <= 1` passes and the commit rule `count(files, "src/**") <= 1` refuses:
```
$ mt check
ok: 1 task(s) valid
$ mt check --commit-msg /tmp/rvw-msg
error: /tmp/rvw-msg: commits.rules.one-file: one source file
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/git_state.rs").

== TO-BE

- 🟢 A pure rename counts as one file in both `count(files, …)` and `changed()`.
- 🟢 A project names a path set once and both calls read it, so `.mindtape/config.toml` holds the exclude list once.


== Changes

- `crates/mindtape/src/git_state.rs`: one rename rule `PathFilter` counting shares.
- `crates/mindtape/src/commit_expr.rs`: count a rename as its destination.
- `crates/mindtape/src/filter_predicates.rs`: read a named path set in `changed()`.
- `.mindtape/config.toml`: declare the source path set once and use it in both rules.


== Rollback

- Revert the commit; renames count twice in commit rules and the lists are copied again.


== Out of scope

- `has(files)` and `matches(files, …)`, which should still see both sides of a rename.


== Dogfood

This repo's `src-budget` and `tweak-stays-one-file` read one path set and agree on a rename.
