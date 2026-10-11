---
name: mindtape
description: 'Drive MindTape, a file-based task tracker, through its `mt` CLI; each
  task is one Typst file. Use it in any repo with `.mindtape/config.toml` for any
  work on tasks, even when the user does not name the tool: listing, finding, filing,
  blocking, closing, validating or retiring tasks, or "what should I work on". There
  MindTape is the tracker, in place of GitHub issues or TODO lists. In a repo with
  no tracker, offer `mt init` first.'
---
<!-- Generated from docs/src/skill.typ by `just docs md`. Do not edit; edit the source. -->

# MindTape (`mt`)

Each task is one Typst file, named by its filename stem. A `links:` target is the path to the other task’s file from the linking file, such as `"../auth/login.typ"`. `mt link` takes a stem and writes that path.

Change tasks with `mt` commands, not by editing files. `mt` fills in the slug, the status date and the injected fields, resolves links, and writes nothing if the result breaks a rule. Edit a file by hand only for what no command writes: the body, a framework priority like `ice(...)`, `estimate`, and an `x-<ns>` slot. Follow `references/hand-editing.md`, then run `mt check`.

## Is this a MindTape project?

It is when `.mindtape/config.toml` exists. Run `mt` in the directory that holds `.mindtape/` or below it. For another project, pass `-w <path>`, or put an alias that `mt worktrees` lists, such as `@name`, right after `mt`. In a repo with no tracker, offer `mt init`.

## Commands

| Goal | Command |
| --- | --- |
| List the default view | `mt ls` |
| List every task | `mt ls ''` (there is no `--all`) |
| Filter and sort | `mt ls '<filter>' --sort <field>:desc` |
| Search text | `mt find <word…>` |
| Read one task | `mt eval <task>` |
| File a task | `mt add <title…>` |
| Set one field | `mt set <task> <field>=<value>` |
| Change status | `mt <status> <task…>` |
| Link tasks | `mt link <task> <kind> <target…>` |
| Rename a task | `mt mv <task> <new-stem>` |
| Retire a task | `mt rm <task…>` |
| Validate | `mt check` |

Run `mt <cmd> --help` for every flag. Read the project’s statuses, link kinds, priority scale and tags from `mt config show`, which prints the merged config (`--origin` names the pack or file each rule comes from), or the statuses alone from `mt flip --list`. Never guess them. For what no `--help` covers, read [https://mlavrinenko.github.io/mindtape/llms.txt](https://mlavrinenko.github.io/mindtape/llms.txt), which lists every docs page as Markdown.

A `<task>` is a stem, a path, or the stem hash `mt ls` prints in its `slug#` column. A stem hash is the same in every listing and changes only when `mt mv` renames the stem. `mt` refuses a hash that several tasks share. When you mention a task to a person, link its file path.

## Finding tasks

Bare `mt ls` applies the first `[[ls.rules]]` entry whose condition holds, else the project’s default view, and its `applied:` footer names which. A filter argument replaces only the filter. The sort and columns of that rule or view still apply. `-n N` keeps the first N rows after the sort.

Common filters are `ready` (open, with every `depends-on` target finished and every `waits-on` checker answering that the wait is over), `blocked`, `status == "wip"`, `has_tag("t")`, `contains("text")` and `due < now()`. `mt ls @<view>` applies a view that `mt views` lists. Before you edit a file, run `mt ls 'touches("<path>")'` to read the tasks named in the `Refs:` trailers of commits that changed it. For every field and function, read `references/filters.md`.

## Filing tasks

`mt add <title…>` makes the filename from the title, writes the task under the first root, and checks it before writing. `mt add --list` lists the templates, and `mt add --describe` lists a template’s slots. Pass a value you type as `--set key=value`. Pass values you generate as YAML on stdin with `-f -`, which needs no shell quoting. Add `--print` to see the task without writing it.

A key names a template slot, which need not be a field. A framework priority, for example, takes its inputs (`--set impact=8`). `mt add` refuses an unknown slot and lists the known ones. When the template has no slot for a field, file the task first, then write the field with `mt set`. A `text` slot escapes Typst markup, so a code span in it lands as literal backticks. Before you fill an unfamiliar template, read `references/add-slots.md`.

`mt import` files a batch from `mt ls --json`’s schema. It takes each stem from the record’s `id`, else from its title. Read `references/import.md` before you run it.

## Changing status

`mt <status> <task…>` is short for `mt flip <status> <task…>`. It stamps the status with today’s date, and `-m "…"` sets the status note. A refused flip names its reason: the legend does not allow the transition, the task has an unfinished dependency, or it is a parent with an unfinished child. Fix the reason. `--force` overrides the refusal and waives a failing `[[on.flip]]` gate. Use it only to correct a mistake. A gate runs its command with `sh -c` in `mt`’s environment, so exit 127 means `mt` ran outside the project’s dev shell. Run the flip inside that shell.

To see what each blocked task waits on, run `mt ls blocked --columns title,why`. A legend may have no `blocked` status, so check `mt flip --list` before you use one. To record an external blocker, run `mt config show` and look for `[[tasks.waits]]` checkers. If one fits, add the condition with `mt set <task> waits-on+=<checker>:<id>`, and the task stays blocked until that checker exits 0. Otherwise, file a task for the blocker and link to it with `depends-on`, or leave the task in a status before work starts with the reason as its note. Do not mark a blocked task `wip`: that counts against any work-in-progress cap and tells readers someone is working on it.

## Commits and checks

A project with the `flip-with-work` commit rule refuses a commit that holds only a `wip` or `done` flip. Leave `mt wip <stem>` uncommitted and commit it with the first change of the work. Commit `mt done <stem>` with the last change, whose `Refs:` trailer names the task. A commit that files a task holds no flip and no `Refs:` trailer.
