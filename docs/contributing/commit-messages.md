<!-- Generated from docs/src/contributing/commit-messages.typ by `just docs md`. Do not edit; edit the source. -->

# Commit messages

The rules are in CONTRIBUTING.md’s Commit Messages section. This page tells you what checks them and why. A work commit looks like this:

```
feat(filter): evaluate expressions over live tasks

The body says what changed and why, in plain prose.

Refs: <stem>
```

## What checks them

`.githooks/commit-msg` runs `mt check --commit-msg`, which applies the `[[commits.rules]]` of the `mindtape-tasks` pack (`crates/mindtape/src/packs/mindtape-tasks.toml`).

- `work-names-a-task` refuses a commit with no `Refs:` trailer, unless every file it changes is a `.typ` file under a task root. This repo’s only task root is `truth/tasks/`, so a change to `.mindtape/config.toml` or any other file needs `Refs:`.
- `flip-with-work` checks each task a commit moves into `wip` or `done`. The commit’s `Refs:` must name the task, and the commit must change a file outside the task root. A task tagged `decision` is exempt from the second half, because a decision has no code to commit with it.

`.mindtape/config.toml` adds one rule of its own:

- `refs-task-is-started` refuses a commit whose `Refs:` task is neither `wip` nor terminal in the staged tree. A task still `proposed` fails it, and so does a flip you left unstaged. The rule refuses early because the repair afterwards means rewriting commits. A decision that closes in a task-only commit passes, since its staged status is `done`.

`just integrate <branch>` checks the other half before it reads the gate record. `.just/scripts/integrate.sh` refuses a branch whose commits name a task that’s still `wip` at the branch tip, and prints the stems. So you put the `done` flip in the task’s last commit.

`mt check --commit-msg` also refuses a `Refs:` value that’s a path or names no task.

## Why Refs names a stem

A task’s stem is its identity. A path breaks when the task moves to another folder, and the stem doesn’t. Before you edit a file, run `mt ls 'touches("<path>")'` to list the tasks whose commits changed it. They tell you what shaped the file.
