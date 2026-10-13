// docs/src/contributing/commit-messages.typ renders
// docs/contributing/commit-messages.md through docs/generate-md.sh (typlite;
// docs/src/lib.typ lists the constraints).
//
// What checks the commit rules and why they have the shape they do. The
// rules are in docs/src/contributing.typ's Commit Messages section.
#import "../lib.typ": doc-title, just

#metadata((
  output: "docs/contributing/commit-messages.md",
)) <mindtape-md>

#doc-title[Commit messages]

The rules are in CONTRIBUTING.md's Commit Messages section. This page tells
you what checks them and why. A work commit looks like this:

// docs:hand-typed a message shape with a placeholder stem; no commit holds it
```
feat(filter): evaluate expressions over live tasks

The body says what changed and why, in plain prose.

Refs: <stem>
```

= What checks them

`.githooks/commit-msg` refuses a message that names your `$HOME` or the
checkout, as the pre-commit hook refuses such a file
(#link("docs/contributing/gates.md")[docs/contributing/gates.md] says why). Then it runs `mt check --commit-msg`, which applies the
per-commit `[[commits.rules]]` of the `mindtape-tasks` pack
(`crates/mindtape/src/packs/mindtape-tasks.toml`).

- `work-names-a-task` refuses a commit with no `Refs:` trailer, unless every
  file it changes is a `.typ` file under a task root. This repo's only task
  root is `truth/tasks/`, so a change to `.mindtape/config.toml` or any other
  file needs `Refs:`.
- `flip-with-work` checks each task a commit moves into `wip` or `done`. The
  commit's `Refs:` must name the task, and the commit must change a file
  outside the task root. Two commits are exempt from the second half: one
  that retires another task with `mt rm`, and one for a task tagged
  `decision`, which has no code to commit with it.
- `named-tasks-finish-at-the-tip` is a `scope = "tip"` rule, which runs only
  under `mt check --commits <range>`. It refuses a range whose commits name a
  task that's still `wip` in the newest commit, and prints the stems. So you put
  the `done` flip in the task's last commit. The `pre-merge` hook of #just.land() runs
  `mt check --commits main..HEAD` to hold a branch to it.

`.mindtape/config.toml` adds one rule of its own:

- `refs-task-is-started` refuses a commit whose `Refs:` task is neither `wip`
  nor terminal in the staged tree. A task still `proposed` fails it, and so
  does a flip you left unstaged. The rule refuses early because the repair
  afterwards means rewriting commits. A decision that closes in a task-only
  commit passes, since its staged status is `done`.

`mt check --commit-msg` also refuses a `Refs:` value that's a path or names
no task.

= Why Refs names a stem

A task's stem is its identity. A path breaks when the task moves to another
folder, and the stem doesn't. Before you edit a file, run
`mt ls 'touches("<path>")'` to list the tasks whose commits changed it. They
tell you what shaped the file.
