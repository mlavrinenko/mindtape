// Source for skills/mindtape/SKILL.md, rendered by docs/generate-md.sh with
// typlite. An agent loads SKILL.md whole every time the skill triggers, so it
// keeps only what changes what the agent does and what no `--help` output
// says. Detail an agent needs only for some tasks goes in
// skills/mindtape/references/, which this file links.
//
// `frontmatter` stays a plain string, because typlite cannot emit YAML and the
// generator writes it with `yq`. The title's backticks are interpolated from a
// string, never written as a raw span, so the rendered h1 stays plain text and
// the generator turns it into a `#` heading.
#import "lib.typ": doc-title

#metadata((
  output: "skills/mindtape/SKILL.md",
  frontmatter: (
    name: "mindtape",
    description: "Drive MindTape, a file-based task tracker, through its `mt` CLI; each task is one Typst file. Use it in any repo with `.mindtape/config.toml` for any work on tasks, even when the user does not name the tool: listing, finding, filing, blocking, closing, validating or retiring tasks, or \"what should I work on\". There MindTape is the tracker, in place of GitHub issues or TODO lists. In a repo with no tracker, offer `mt init` first.",
  ),
)) <mindtape-md>

#doc-title[MindTape (#"`"mt#"`")]

Each task is one Typst file, and the filename stem is the task's identity. A
`links:` target is the path to the other task's file from the linking file,
such as `"../auth/login.typ"`. `mt link` takes a stem and writes that path.

Make every change through `mt`. It writes the slug, the status date and the
fields the config injects, resolves link targets, and checks the result
against the config. When the check fails, `mt` writes nothing. Hand-edit a task
file only for what no command writes: the body, a framework priority such as
`ice(...)`, and the `estimate` dict. Follow `references/hand-editing.md`, then
run `mt check`.

= Is this a MindTape project?

It is when `.mindtape/config.toml` exists. Run `mt` in the directory that
holds `.mindtape/` or below it. For another project, pass
`-w <path>`, or put an alias that `mt worktrees` lists, such as `@name`, right
after `mt`. In a repo with no tracker, offer `mt init`.

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
  [Retire a task], [`mt rm <task…>`],
  [Validate], [`mt check`],
)

`mt <cmd> --help` lists every flag. Read the project's statuses, link kinds,
priority scale and tags from `mt config show`, which prints the merged config,
or the statuses alone from `mt flip --list`. Never guess them.

A `<task>` is a stem, a path, or the stem hash `mt ls` prints in its `slug#`
column. A stem hash is the same in every listing, and one that several tasks
share is refused. When you mention a task to a person, link its file path.

= Finding tasks

Bare `mt ls` applies the first `[[ls.rules]]` entry whose condition holds,
else the project's default view, and its `applied:` footer names which. A
filter argument replaces only the filter. The sort and columns of that rule or
view still apply. `-n N` keeps the first N rows after the sort.

Common filters are `ready` (open, with every `depends-on` target finished),
`blocked`, `status == "wip"`, `has_tag("t")`, `contains("text")` and
`due < now()`. `mt ls @<view>` applies a view that `mt views` lists. Before you
edit a file, run `mt ls 'touches("<path>")'` to read the tasks named in the
`Refs:` trailers of commits that changed it. `references/filters.md` lists
every field and function.

= Filing tasks

`mt add <title…>` makes the filename from the title, writes the task under the
first root, and checks it before writing. `mt add --list` lists the templates,
and `mt add --describe` lists a template's slots. Pass a value you type as
`--set key=value`. Pass values you generate as YAML on stdin with `-f -`,
which needs no shell quoting. `--print` shows the task without writing it.

A key names a template slot, which need not be a field. A framework priority,
for example, takes its inputs (`--set impact=8`). An unknown slot is an error
that lists the known ones. When the template has no slot for a field, file the
task first, then write the field with `mt set`. A `text` slot escapes Typst
markup, so a code span in it lands as literal backticks. Read
`references/add-slots.md` before you fill an unfamiliar template.

`mt import` files a batch from `mt ls --json`'s schema. It takes each stem from
the record's `id`, else from its title, so read `references/import.md` before
you run it.

= Changing status

`mt <status> <task…>` is short for `mt flip <status> <task…>`. It stamps the
status with today's date, and `-m "…"` sets the status note. A refused flip
names its reason: the legend does not allow the transition, the task has an
unfinished dependency, or it is a parent with an unfinished child. Fix the
reason. `--force` overrides the refusal and waives a failing `[[on.flip]]`
gate, so use it only to correct a mistake.

An `[[on.flip]]` gate runs its command with `sh -c` in `mt`'s environment.
Exit 127 means the shell did not find the command, which happens when `mt`
runs outside the project's dev shell. Run the flip inside that shell.

`mt ls blocked --columns title,why` shows what each blocked task waits on. A
legend may have no `blocked` status, so check `mt flip --list` before you use
one. To record an external blocker, file a task for it and link to it with
`depends-on`, or leave the task in a status before work starts with the reason
as its note. Do not mark a blocked task `wip`, because that counts against any
work-in-progress cap and tells readers someone is working on it.

= Commits and checks

A project with the `flip-with-work` commit rule refuses a commit that holds
only a `wip` or `done` flip. Leave `mt wip <stem>` uncommitted and commit it
with the first change of the work. Commit `mt done <stem>` with the last
change, whose `Refs:` trailer names the task. A commit that files a task holds
no flip and no `Refs:` trailer.

`mt config show --origin` shows which pack or project file each rule comes
from, so you know where to change one.
