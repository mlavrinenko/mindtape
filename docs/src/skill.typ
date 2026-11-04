// Source for skills/mindtape/SKILL.md, rendered by docs/generate-md.sh with
// typlite. An agent loads SKILL.md whole whenever the skill triggers, so it
// keeps only what changes what the agent does and no `--help` says. Detail
// for some tasks goes in skills/mindtape/references/, which this file links.
// `frontmatter` stays a plain string, because typlite cannot emit YAML and the
// generator writes it with `yq`. The title's backticks come from a string, not
// a raw span, so the rendered h1 stays plain text.
#import "lib.typ": doc-title, mindtape, www-llms
#import "/www/utils/set-command-notes.typ": set-command-list

#metadata((
  output: "skills/mindtape/SKILL.md",
  frontmatter: (
    name: "mindtape",
    description: "Drive "
      + mindtape
      + ", a file-based task tracker, through its `mt` CLI; each task is one Typst file. Use it in any repo with `.mindtape/config.toml` for any work on tasks, even when the user does not name the tool: listing, finding, filing, blocking, closing, validating or retiring tasks, or \"what should I work on\". There "
      + mindtape
      + " is the tracker, in place of GitHub issues or TODO lists. In a repo with no tracker, offer `mt init` first.",
  ),
)) <mindtape-md>

#doc-title[#mindtape (#"`"mt#"`")]

Each task is one Typst file, named by its filename stem. A `links:` target is
the path to the other task's file from the linking file, such as
`"../auth/login.typ"`. `mt link` takes a stem or a stem hash and writes that
path.

Change tasks with `mt` commands, not by editing files. `mt` fills in the slug,
the status date and the injected fields, resolves links, and writes nothing if
the result breaks a rule. Edit a file by hand only for what no command writes:
the body, a framework priority like `ice(...)`, `estimate`, and an `x-<ns>`
slot. Follow `references/hand-editing.md`, then run `mt check`.

= Is this a #mindtape project?

It is when `.mindtape/config.toml` exists. Run `mt` at or below the directory that
holds `.mindtape/`. For another project, pass `-w <path>`, or put a name
that `mt at` lists, such as `@name`, right after `mt`. `mt at <name>` shows
whether a name is one project or a set. On a set, #set-command-list() read
every project, and a bare `mt @name` runs `board`. Writes and other commands
refuse a set: name one project instead.

= Commands

#table(
  columns: 2,
  table.header([Goal], [Command]),
  [List the default view], [`mt ls`],
  [List every task], [`mt ls ''` (there is no `--all`)],
  [Filter and sort], [`mt ls '<filter>' --sort <field>:desc`],
  [Search text], [`mt find <word…>`],
  [Read one task], [`mt eval <task>`],
  [File a task], [`mt add <title…>`],
  [Set one field], [`mt set <task> <field>=<value>`],
  [Change status], [`mt <status> <task…>`],
  [Link tasks], [`mt link <task> <kind> <target…>`],
  [Rename a task], [`mt mv <task> <new-stem>`],
  [Retire a task], [`mt rm <task or folder…>`],
  [Validate], [`mt check`],
  [List the rules], [`mt rules`],
  [List the tags], [`mt tags`],
)

Run `mt <cmd> --help` for every flag. Never guess the project's statuses, link
kinds, priority scale or tags: read them from `mt tags`, `mt config show` or
`mt flip --list`. Read a dictionary's words, such as the verbs a title may
start with, from `mt dicts <name>`. Before you flip or commit, run
`mt tags --for <step>` and `mt rules --for <step>`. For more, read #www-llms,
every docs page as Markdown.

A `<task>` is a stem, a path, or the `slug#` hash `mt ls` prints. The hash
changes only when `mt mv` renames the stem; `mt` refuses one shared by several
tasks. Link a task's file path when you name it to a person.

= Finding tasks

Bare `mt ls` applies the first `[[ls.rules]]` entry whose condition holds,
else the project's default view, and the line under the table names which, as
in `default view · 3 of 12 tasks`. A filter argument replaces only the
filter; that rule's or view's sort and columns still apply. `-n N` keeps the
first N rows after the sort.

Common filters are `ready` (open, with every `depends-on` target finished and
every `waits-on` checker answering that the wait is over), `blocked`,
`status == "wip"`, `has_tag("t")`, `contains("text")` and `due < now()`. `mt ls @<view>` applies a view that `mt views` lists. Before you
edit a file, run `mt ls 'touches("<path>")'` to read the tasks named in the
`Refs:` trailers of commits that changed it. For every field and function,
read `references/filters.md`.

= Filing tasks

`mt add <title…>` names the file, checks the task, and writes it under the
first root. Before you file, run `mt tags --for add` for the tags the shape
rules read, including the `no-repro` and `no-ui-ux` opt-outs. Write `x-goals`
with `verdicts((secure: helps[why],), "fast")`: the task's verdicts, then
every neutral goal by name. A goal named both ways fails. Start every Changes
bullet with one backticked path; `derived.changes` is that list. `mt add --list`
lists templates and `mt add --describe` their slots. Pass a typed value as
`--set key=value`, or YAML on stdin with `-f -`. `--print` shows the task
without writing it.

A slot need not be a field: a framework priority takes its inputs
(`--set impact=8`). `mt add` refuses an unknown slot and lists the known ones.
For a field with no slot, file the task then `mt set` it. A `text` slot
escapes markup, so a code span lands as literal backticks and `mt add` warns
naming the slot. Read `references/add-slots.md` before filling an unfamiliar
template.

Read `references/import.md` before `mt import` files a batch.

= Changing status

`mt <status> <task…>` is short for `mt flip <status> <task…>`. It stamps the
status with today's date, and `-m "…"` sets the note. A refused flip names its
reason: the legend forbids the transition, the task has an unfinished
dependency, or a parent has an unfinished child. Fix the reason.
`--force` overrides the refusal and waives a failing `[[on.flip]]` gate; use it
only to fix a mistake. Exit 127 from a gate means `mt` ran outside the dev
shell, so flip from inside it.

`mt ls blocked --columns title,why` shows what each blocked task waits on. A
legend may have no `blocked` status, so check `mt flip --list` first. For an
external blocker, read `references/blocking.md`. Never mark a blocked task
`wip`: that counts against a WIP cap.

`mt` runs no command from the config, neither a `[[tasks.waits]]` checker, an
`[[on.flip]]` `run` nor a `[go]` or `[add]` editor, until the user allows them.
Until then a wait reads `unknown: not allowed`, a flip gate refuses with
`gate not allowed`, an editor falls back to `$VISUAL`, then `$EDITOR`, and `mt`
warns once with the command list. Never run `mt allow`, or `--force` past that refusal: the
commands run with the user's rights. Pass the warning to the user.

= Commits and checks

A project with the `flip-with-work` commit rule refuses a commit that holds
only a `wip` or `done` flip. Commit `mt wip <stem>` with the first change of
the work and `mt done <stem>` with the last, whose `Refs:` trailer names the
task. A commit that files a task holds no flip and no `Refs:` trailer.

If `mt` says `pack source not installed`, run `mt packs install`.
