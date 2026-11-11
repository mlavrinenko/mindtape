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

Change tasks with `mt` commands, not by hand: `references/hand-editing.md` covers the few fields no command writes.

## Is this a Mindtape project?

It is when `.mindtape/config.toml` exists. Run `mt` at or below the directory that holds it, or pass `-w <path>`. Run `mt explain` first: it names this project’s rules, tags, views and dicts. `mt <cmd> --help` lists every flag, and `mt explain rules --for <step>` or `mt explain tags --for <step>` lists what an add, flip or commit checks, and `mt explain add` lists the `[[on.add]]` hooks that fill a new task or choose its folder.

## What the config runs

`mt` runs nothing from the config until the user allows it: not a `[[tasks.waits]]` checker, an `[[on.flip]]` `run`, or a `[go]` or `[add]` editor. Until then, a wait reads `unknown: not allowed`, a flip gate refuses with `gate not allowed`, and an editor falls back to `$VISUAL`, then `$EDITOR`. `mt` also prints one warning that lists the commands it did not run. Never run `mt allow`, and never pass `--force` past that refusal or a failing gate: those commands run with the user’s rights. `--force` also waives a failing `[[on.flip]]` gate, so use it only to fix a mistake. Pass that warning to the user.

## References

- `references/filters.md`: the filter language, and how a listing picks its rule or view.
- `references/hand-editing.md`: a task’s name, and the body, priority, `estimate` and `x-<ns>` fields no command writes.
- `references/add-slots.md`: `mt add` slots and their value kinds.
- `references/import.md`: a batch `mt import`.
- `references/blocking.md`: an external blocker, and a blocked task’s status.
