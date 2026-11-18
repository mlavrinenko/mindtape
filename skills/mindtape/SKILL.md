---
name: mindtape
description: 'Drive Mindtape, a file-based task tracker, through its `mt` CLI; each
  task is one Typst file. Use it in any repo with `.mindtape/config.toml` for any
  work on tasks, even when the user does not name the tool: listing, finding, filing,
  blocking, closing, validating or retiring tasks, or "what should I work on". In
  those repos, use it instead of GitHub issues or TODO lists. In a repo with no tracker,
  offer `mt init` first.'
---
<!-- Generated from docs/src/skill.typ by `just docs md`. Do not edit; edit the source. -->

# Mindtape (`mt`)

Change tasks with `mt` commands, not by hand. `references/hand-editing.md` covers the few fields that need a manual edit.

A project holds `.mindtape/config.toml`. Run `mt` at or below that directory, or pass `-w <path>`. `mt <cmd> --help` lists every flag.

## The recipe

1. `mt explain` prints the project’s shape: its roots, statuses, templates and rules, and the fields a task takes.
2. `mt ls` lists what the default view picks and names the filter it applied. `mt find <words>` lists the tasks that contain every word.
3. `mt add --describe` lists the template’s slots and the `[[on.add]]` hooks that fill a new task or choose its folder. `mt explain templates` lists the templates and the `kind` each sets.
4. `mt add` files the task.
5. `mt check` checks every task against the project’s rules.
6. Before a write, `mt explain rules --for <moment>` lists what that moment checks, and `mt explain tags --for <moment>` lists the tags those rules read. The moments are `add`, `commit`, `flip` and `rm`.

## Commands the config runs

`mt` runs nothing from the config until the user allows it: not a `[[tasks.waits]]` checker, an `[[on.flip]]` or `[[on.rm]]` `run`, or a `[go]` or `[add]` editor. Until then, a wait reads `unknown: not allowed`, a flip gate refuses with `gate not allowed`, and an editor falls back to `$VISUAL`, then `$EDITOR`. `mt` also prints one warning that lists the commands it did not run. Never run `mt allow`, and never pass `--force` past that refusal or a failing gate: those commands run with the user’s rights. `--force` also waives a failing `[[on.flip]]` gate, so use it only to fix a mistake. Pass that warning to the user.

## References

- `references/filters.md`: the filter language, and how a listing picks its rule or view.
- `references/hand-editing.md`: a task’s name, and the body, priority, `estimate` and `x-<ns>` fields no command writes.
- `references/add-slots.md`: `mt add` slots and their value kinds.
- `references/import.md`: a batch `mt import`.
- `references/blocking.md`: an external blocker, and a blocked task’s status.
