#import "../utils/page.typ": page
#import "../utils/ui.typ": capture-line, terminal
#import "../utils/site.typ": u
#import "../utils/slot-kind-notes.typ": slot-kind-notes

#let slot-kinds = json("../generated/slot-kinds.json")

#let meta = (
  route: "/templates/",
  label: "Templates & slots",
  title: "Templates & slots",
  summary: "How `mt add` files a task from a template, and how you or an agent fill its `slot()` calls.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

You can write a task file by hand, but `mt add` saves you the boilerplate. It
writes a new task from a template. Every task then starts with the same fields
and sections. A template is a `.typ` file with the same fields as a task, and
its values are `slot(...)` calls. It is in a template directory rather than a
task root. #link(u("/concepts/#which-files-count-as-tasks"))[Which files count as tasks]
says why a template is not a task source. `mt add` fills the slots, sets the
status, checks the task against your rules and writes it only if it passes. If
`mt add` refuses, it doesn't leave a file behind.

The examples below run against `examples/api-service/`, a sample backend
project.

= Write a template

`mt add` reads templates from `.mindtape/templates/`, or from the directories
in `[templates] dirs`. It uses `task.typ` unless you pick another with
`--template <name>`. To change that default name, set `[templates] default`.
If the project has no such file, `mt add` uses a built-in template whose only
slot is `title`. `mt templates` lists the templates it finds, with the
`kind` each sets and its slot count. A
#link(u("/packs/#a-pack-in-a-directory"))[directory pack] can include templates too. They
come after your own directories, so a template of yours with the same name
wins.

Sometimes `mt add` refuses a task because of one field, and another template
would have set that field the way you wanted. In that case the error suggests
that template:
#capture-line("add-template-hint", "^hint: "). Each matching
template gets its own line.

The template in `examples/api-service/.mindtape/templates/task.typ`:

#raw(read("../generated/api-service-template.typ"), lang: "typ", block: true)

Its `as-is` slot takes one plain line that carries the essence, then the
numbered steps. The line is not a bullet, not a marker and not a fragment, and
it ends with a period or a colon. `mt check` refuses an `AS-IS` that opens on a
step, because the point belongs before the commands.

A slot can stand for a field value, a value inside a call such as a framework
input, or a section of the body. `slot(name, ...)` takes these optional
arguments:

#table(
  columns: 2,
  table.header([Argument], [What it does]),
  [`kind`],
  [The value's kind. #slot-kinds.kinds.map(k => [#raw(k.name) is #slot-kind-notes.at(k.name)]).join([, ], last: [ or ]).],

  [`choices`],
  [The values a `choice` or `multichoice` slot accepts. If a slot is a whole field value, as in `kind: slot("kind", example: "test")`, and has no `choices` of its own, it takes the field's allowed values: `values` from `[tasks.fields.kind]`, `allowed` from `[tasks.assignee]`. A field that holds a list makes the slot a `multichoice`.],

  [`default`], [The value used when nothing fills the slot.],
  [`example`],
  [A sample value that `--describe` and the editor scaffold show, and `mt check` fills. `mt add` never files it.],

  [`required`],
  [`false` lets the slot stay empty. A slot with a `default` is never required.],

  [`prompt`], [A hint that `--describe` and the editor show.],
  [`env`], [An environment variable that fills the slot.],
)

A `text` slot escapes its value, so `#`, `*`, `_`, `[` and the other markup
characters print as typed. A `raw` slot writes its value as Typst markup,
and `mt add` refuses a value that doesn't parse as markup. Use `raw` for body
sections that hold lists and code, like the template above.

A backtick in a `text` slot can mean a code span, but `text` escaping
writes it as a literal backslash. `mt add` still writes the task, but warns and
names the slot:

#terminal("add-text-slot-span")

Only the backtick warns. A path's `_` or a plain `a < b` is literal text, and
`mt add` writes it without a warning.

The `default` and `example` of a `text` or `raw` slot can be a raw block as
well as a string, so a body section's example reads as markup, as in the
template above. Typst removes the indentation every line shares, the closing
fence's line included, and ignores the language tag.

A raw slot can stand for a whole body section, heading included. This
repository's templates put one `goals` slot after `Dogfood`, and its value
carries the `== Goals` heading with every goal verdict. The template body
leaves the heading to the slot:

// docs:hand-typed the slot shape a project copies before it has a goal registry
````typ
#slot(
  "goals",
  kind: "raw",
  required: true,
  example: ```typ
  == Goals

  + #Helps(adopter, to: guide-the-implementer)[why].
  + #Neutral(read-tasks-at-a-glance, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).
  ```,
)
````

`required: true` matters because an optional raw slot with no default fills as
the literal `none`. `mt add` refuses a blank required slot. The author then
rates every goal, by hand in the editor or with `--set goals=...`.

= Check the templates

`mt check` checks every template `mt add` accepts. So if a config
change breaks a template, you find out from `mt check` right away, not at the
next `mt add`.

It fills each template as `mt add` would, with every slot set to its
`example`, else its `default`. Then it checks the result like a task, alone,
against an empty backlog and an empty working tree, under the file name
`mt add` would give it. A rule that reads `changed()` doesn't see any changes.
A budget on the work in progress then never fires on a template. A problem the
template can fix counts, and the error reports the template file and the line
of the field:
#capture-line("check-template-stale", "^error: .*bug.typ").

With `[templates] check-defaults = true`, it also fills each template with
every slot's `default`, falling back to its `example` for a slot with no
default. A default that falls behind its example then fails the check too. `mt add`
catches it before filing the task. The option is off by default because a template may
leave a goal `neutral` so `mt add` refuses a task that helps no goal until the
caller answers one. A slot with no default, such as the `goals` slot, has the
same value in both variants. The option then leaves that slot alone. `mt check`
reports a breach that both variants share once.
A problem about the backlog or the file's place doesn't count. That covers a
`count()` cap, a rule on `ready`, `blocked`, `path` or `dir`, a link cycle, and
a link to a task that isn't there.

A slot with neither an `example` nor a `default` leaves the field empty.
`mt check` then reports the slot and doesn't check anything else in that
template. When
`mt add` adds a slot for a required field, the slot's example comes from
config: the `example` key of `[tasks.fields.<name>]`, `[tasks.assignee]`,
`[tasks.estimate]`, `[tasks.priority]` or `[tasks.difficulty]`, else the
field's `default`. If a required field has neither, nothing can fill the slot,
and `mt check` names the table that should have the key. A `multiple` field
takes a list, and an estimate takes a table such as `{ days = 1 }`. A
framework input keeps the example its range gives. `mt check` reports a
template that doesn't evaluate once, with the Typst error.

A `text` slot's example can't show a code span. `mt add` escapes the
backticks, so anyone who copies the example's style files literal backslashes.
`mt check` names the slot, and you either make it `raw` or drop the backticks.

`mt check` reports the built-in template as `built-in template task`, at the severity
`[templates] fallback-severity` sets: `"warning"` by default, or `"error"`.
Any #link(u("/config/#layers"))[config layer] can set it, the global
`defaults.toml` included. Your project's own templates always report
errors.

= See the slots

To see what a template requires, `mt add --describe` lists each slot with its
kind, whether it's required, the values a `choice` slot accepts, and its
prompt. When the project declares `[[on.add]]` hooks, a blank line and a
`hooks` heading separate them from the slots:

#terminal("api-service-describe")

`--describe=json` prints the same thing for a script or an agent, in one
object whose `slots` key holds the array and whose `hooks` key lists the
project's #link(u("/hooks/#add-hooks"))[`[[on.add]]`] hooks.

= Fill the slots from values

#table(
  columns: 2,
  table.header([Source], [Example]),
  [A values file, YAML or JSON], [`mt add -f values.yaml`],
  [Values on stdin], [`mt add -f -`],
  [A token], [`mt add --set difficulty=2`, or `mt add -- difficulty=2`],
  [Title words], [`mt add Log slow database queries`],
  [An environment variable], [`MINDTAPE_ADD_DIFFICULTY=2 mt add`],
  [The slot's `default`], [`slot("difficulty", kind: "int", default: 1)`],
)

When two sources set the same slot with `=`, the later one wins. A later `+=`
appends to what the earlier source set. `mt add` reads `-f` files first, left
to right, then `--set` tokens, then the tokens after `--`. Title words,
environment variables and defaults fill only the slots still empty, in that
order. A slot reads the variable its `env` names, or else `MINDTAPE_ADD_` plus
its name in capitals, with `-` turned into `_`.

A token takes one of three forms:

#table(
  columns: 2,
  table.header([Token], [Effect]),
  [`key=value`], [Sets the slot.],
  [`key+=value`], [Appends to a `list` slot. Repeat it for more items.],
  [`key@path`], [Reads the value from a file, dropping its trailing newlines.],
)

The first `=` or `@` in a token ends the slot name, so `title=a@b` sets the
title `a@b`. A list slot takes one value as a one-item list, so `tags=bug`
files that single tag. A title word that starts with `+`, as in
`mt add Fix login +auth`, goes
to the `tags` slot if the template has one. If you name neither a slot nor a
core field, `mt add` refuses and points at `mt add --describe`.

A token may also set a core field the template has no slot for.
`mt add --set due=2026-12-01` inserts `due` in the filed task, and
`--set tags+=bug` appends to the tags the template already writes, or to a
`tags` slot's `default`, or inserts the list when the template writes none.
These are the fields `mt set` writes: `title`,
`priority`, `difficulty`, `kind`, `assignee`, `waits-on`, `tags`, `due` and
`start`. `mt add` refuses a `+=` on a single-valued field. `dir` is not a field:
`mt add --set dir=…` refuses and points at `--dir`.

When the template fills a core field through a whole-value slot under another
name, as in `kind: slot("type")`, a core-field token or flag fills that slot.
The slot name and the core name count as one field, so `mt add` refuses a
direct flag plus a `--set` token for either spelling. A positional title fills an
aliased title slot the same way, and the file stem still comes from the title.

The scalar fields also have direct flags: `--due`, `--start`, `--assignee`,
`--kind`, `--priority` and `--difficulty`. Each is an alias for
`--set <field>=<value>`, so it appears in `mt add --help`. `mt add` refuses a
field named with both a flag and a `--set` token.

= Values from a file

For long values, use `-f`. A YAML block scalar (`|`) keeps line breaks and
doesn't need shell quoting, so a `raw` section reads the same here as in the task.
`values.yaml`:

#raw(read("../generated/api-service-values.yaml"), lang: "yaml", block: true)

`mt add -f values.yaml --print` checks the task and prints it without
writing it:

#raw(read("../generated/api-service-add-print.typ"), lang: "typ", block: true)

A `.json`, `.yaml` or `.yml` extension picks the parser. For stdin and any
other name, `mt add` reads JSON if the text starts with `{` or `[`, and YAML
otherwise. You can pass `-f` more than once. A later file's value replaces an
earlier one, and `null` clears it.

= What `mt add` sets for you

- `status`: the first status in the #link(u("/statuses/"))[legend] that isn't
  terminal, with today's date. If the template sets `status`, as in
  `status: done(2026, 1, 1)`, `mt add` files the task at that status with
  today's date instead. `--status <name>` picks another.
- `id`: a new UUID, only when `[tasks.id] format` is `uuid-v4` or `uuid-v7`.
  Otherwise the file name is the task's identity.
- Required fields the template leaves out. If the config requires
  `priority`, `difficulty`, `kind`, `assignee`, `estimate` or an `x-<ns>`
  field and the template doesn't set it, `mt add` adds a slot for it. A
  project that sets `[tasks.estimate] required = false`, as this repo and
  `examples/api-service` do, doesn't get an estimate slot. The slot
  shows up in `--describe` and the editor. The `assignee` slot reads the
  `MINDTAPE_PRINCIPAL` environment variable. When a requirement depends only
  on the status, `mt add` checks it against the status it files the task at.
  So `--status done` doesn't add a slot for a field with `required = "!terminal"`.
  If the priority must come from a framework, you get one slot per framework
  input, as #link(u("/priority/"))[Priority] describes.

= The destination file

`mt add` writes to the first
#link(u("/concepts/#a-task-is-a-file"))[task root] and names the file after
the title: `log-slow-database-queries.typ`. `--name <stem>` sets the name.
`--dir <path>` writes to another directory. A relative path resolves from the
project root. The same command then lands in the same place from any
subdirectory. An absolute path stays as-is. A folder under a root can
pick up #link(u("/derived-tags/"))[derived tags]. `mt` removed `--into` and
`--to`. `--dir` names the one destination. If a file with that name exists,
`--dedup` decides: `fail`, `skip` (doesn't write a file and exits 0) or `add-suffix`
(write `<name>-2.typ`).

A project's #link(u("/hooks/#add-hooks"))[`[[on.add]]`] hooks may fill fields
and choose the destination. An explicit `=` value (a `--set`, a positional
title or an edit in `--edit`) wins over a hook `set`, `--dir` wins over a hook
`dir`, and `--no-hooks` skips the hook list entirely. An explicit `+=` appends
and leaves a later hook `set` free to replace the list.

= Fill the slots in an editor

`mt add --edit` opens your editor on a YAML file with one entry per slot,
filled with the values you passed. Save and close it, and `mt add` reads it
back. In a terminal, `mt add` opens the editor by itself if a required slot is
still empty. Set `[add] interactive` to `always` or `never` to change that.
Without a terminal on stdin and stdout, `mt add` doesn't open an editor, and a
required slot left empty fails the add with the slot's name. That's the case
for a script or an agent.

Above each entry, a comment gives the slot's prompt, its kind and whether it's
required, then its `choices`, its `default` and its `example` where the slot
declares them. A `date` slot spells `YYYY-MM-DD` with a sample, so a wrong
format doesn't read as a valid answer, and a `date` default pre-fills the
entry. A slot you leave untouched keeps its default.

A buffer `mt add` can't use comes back in the same editor, with the error as a
`#` comment above the values. A typo then costs one correction instead of the
whole form. That covers a buffer that isn't a YAML mapping, a value the slot's
kind refuses, and a key the template doesn't declare. It also covers a rule
that refuses the filled task. Every breach gets its own `#` line. An editor
that exits non-zero ends the loop and keeps the buffer, naming the file it
kept. An editor that saves the same buffer again ends it too. `mt` parks the
task and prints the file's path.

The editor is `[add] editor`, else `$VISUAL`, else `$EDITOR`, else `vi`
(`notepad` on Windows). A project's `[add] editor` runs only after
#link(u("/cli/allow/"))[`mt allow`]. `[add] header` replaces the instructions at the top of
the file, and `[add] comments = false` leaves out every scaffold comment. A
refused buffer still carries its error, because that line is the refusal, not
an instruction.

= When `mt add` verifies

`mt add` checks the task against your project's rules before it writes. A
refused add doesn't write a file. `[add] verify-when` decides whether a run
checks, as a #link(u("/cli/ls/#conditional-rules"))[condition] over the run's
own facts: `tty()`, `env("NAME")`, `on_branch("glob")`, `now()` and
`weekday()`. Absent, every run checks. A project that says nothing keeps the
refusal.

// docs:hand-typed the one-line policy a project copies, not a fixture
```toml
[add]
verify-when = "!tty()"
```

`!tty()` checks only a run without a terminal. A person at a terminal files an
incomplete task, and a script or an agent still gets the refusal. `--verify`
and `--no-verify` override the expression for one call.

A run that skips the check leaves an unfilled required slot as its `slot(...)`
call, runs no `[[on.add]]` hook, and writes the file. It ends on a note naming
`mt check`, which lists what the task is missing.
