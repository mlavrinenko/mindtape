// Source for skills/mindtape/SKILL.md, rendered by docs/generate-md.sh with
// typlite. An agent loads SKILL.md whole whenever the skill triggers, so it
// opens with the recipe and keeps only what no command prints: which project
// it applies to, the safety rules around what the config runs, and where the
// references live. Steps live in `mt <cmd> --help` and `mt config show
// --resolved`, detail in skills/mindtape/references/, which this file links.
// `frontmatter` stays a plain string, because typlite cannot emit YAML and the
// generator writes it with `yq`. The title's backticks come from a string, not
// a raw span, so the rendered h1 stays plain text.
#import "lib.typ": doc-title, mindtape

#metadata((
  output: "skills/mindtape/SKILL.md",
  frontmatter: (
    name: "mindtape",
    description: "Drive "
      + mindtape
      + ", a file-based task tracker, through its `mt` CLI; each task is one Typst file. Use it in any repo with `.mindtape/config.toml` for any work on tasks, even when the user does not name the tool: listing, finding, filing, blocking, closing, validating or retiring tasks, or \"what should I work on\". In those repos, use it instead of GitHub issues or TODO lists. In a repo with no tracker, offer `mt init` first.",
  ),
)) <mindtape-md>

#doc-title[#mindtape (#"`"mt#"`")]

Change tasks with `mt` commands, not by hand. `references/hand-editing.md`
covers the few fields that need a manual edit.

A project holds `.mindtape/config.toml`. Run `mt` at or below that directory,
or pass `-w <path>`. `mt <cmd> --help` lists every flag.

= The recipe

1. `mt config show --resolved` prints every key the project sets, with each
  default filled in: its roots, statuses, templates, rules, and the fields a
  task takes.
2. `mt ls` lists what the default view picks and names the filter it applied.
  `mt find <words>` lists the tasks that contain every word.
3. `mt add --describe` lists the template's slots and the `[[on.add]]` hooks
  that fill a new task or choose its folder. `mt templates` lists the
  templates and the `kind` each sets.
4. `mt add` files the task.
5. `mt check` checks every task against the project's rules.

= Commands the config runs

`mt` runs nothing from the config until the user allows it: not a
`[[tasks.waits]]` checker, an `[[on.flip]]` or `[[on.rm]]` `run`, or a `[go]`
or `[add]` editor. Until then, a wait reads `unknown: not allowed`, a flip gate refuses
with `gate not allowed`, and an editor falls back to `$VISUAL`, then
`$EDITOR`. `mt` also prints one warning that lists the commands it did not
run. Never run `mt allow`, and never pass `--force` past
that refusal or a failing gate: those commands run with the user's rights.
`--force` also waives a failing `[[on.flip]]` gate, so use it only to fix a
mistake. Pass that warning to the user.

= References

- `references/filters.md`: the filter language, and how a listing picks its
  rule or view.
- `references/hand-editing.md`: a task's name, and the body, priority,
  `estimate` and `x-<ns>` fields no command writes.
- `references/add-slots.md`: `mt add` slots and their value kinds.
- `references/import.md`: a batch `mt import`.
- `references/blocking.md`: an external blocker, and a blocked task's status.
