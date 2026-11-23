#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Let a task whose only work is a record edit close",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 6,
  ),
  difficulty: 2,
  links: (
    (
      task: "correct-the-stale-goal-migration-claims.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "tooling", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 23),
)

== AS-IS

+ Starting `correct-the-stale-goal-migration-claims`, whose only change is one line of a done task's record
+ 🔴 #raw("crates/mindtape/src/packs/tasks.toml"): `flip-task-with-its-work` refuses a `task` whose whole work edits another task file, so the task can neither start nor close.

The staged tree holds only the two task files, so `tasks_only` is true and the task is `kind: "dev"`:

```
$ mt check --commit-msg msg.txt
error: msg.txt: commits.rules.flip-task-with-its-work: correct-the-stale-goal-migration-claims: start or close a task in a commit that carries its work
```

The two escapes are `staged(kind == "decision")` and `staged(kind == "milestone")`. `mt set kind=decision` then refuses, because the repo's own `require-alternatives-for-design` wants an `Alternatives` section the task has no honest content for.

== TO-BE

- 🟢 A task whose `Changes` name only `truth/tasks/` paths starts and closes in the commit that edits them.
- 🟢 A project that refuses it says so in the rule message and names the kind that fits a record-only task.

```
$ mt done correct-the-stale-goal-migration-claims && git commit
[goal-record <sha>] task: correct the goal migration record's prelude version
```

== Changes

- `crates/mindtape/src/packs/tasks.toml`: let `flip-task-with-its-work` carry an escape for a task whose whole work is a record edit, or name `decision` in its message.
- `crates/mindtape-cli/tests/it/`: a close whose only work is another task's record passes, and a flip that carries no work still fails.
- `www/content/commit-rules.typ`: the escape, and why a record-only task needs one.
- `CHANGELOG.md`: an entry under `[Unreleased]`.

== Out of scope

- The record correction that surfaced this. It rides in its own commit.
- Changing the kind of every record-correction task already filed.

== Dogfood

Every record correction in this repo closes with the task that tracks it, instead of a bare `task:` commit that names none.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[the commit that fixes a record also closes the task that claims it].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
