#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/templates/",
  label: "Templates & slots",
  title: "Templates & slots",
  summary: "How `mt add` files a task from a template, and how you or an agent fill its `slot()` calls.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`mt add` writes a new task from a template, so every task starts with the same
fields and sections. A template is a task file whose values are `slot(...)`
calls. `mt add` fills the slots, sets the status, checks the task against the
project's rules and writes it only if it passes. A refused add leaves no file
behind.

The examples below run against `examples/api-service/`, a backend team's
project.

= Write a template

`mt add` reads templates from `.mindtape/templates/`, or from the directories
in `[templates] dirs`. It uses `task.typ` unless you name another with
`--template <name>`, and `[templates] default` changes that name. When the
project has no such file, `mt add` uses a built-in template whose only slot is
`title`. `mt add --list` lists the templates it finds.

This is the template in `examples/api-service/.mindtape/templates/task.typ`:

#raw(read("../generated/api-service-template.typ"), lang: "typ", block: true)

A slot can stand for a field value, a value inside a dictionary such as
`estimate`, or a section of the body. `slot(name, ...)` takes these optional
arguments:

#table(
  columns: 2,
  table.header([Argument], [What it does]),
  [`kind`],
  [What the value is: `string` (the default), `int`, `float`, `real`, `bool`, `date` (`YYYY-MM-DD`), `list` (of strings), `choice`, `text` or `raw`.],

  [`choices`], [The values a `choice` slot accepts.],
  [`default`], [The value used when nothing fills the slot.],
  [`required`],
  [`false` lets the slot stay empty. A slot with a `default` is never required.],

  [`prompt`], [A hint that `--describe` and the editor show.],
  [`env`], [An environment variable that fills the slot.],
)

A `text` slot escapes its value, so `#`, `*`, `_`, `[` and the other markup
characters print as typed. A `raw` slot pastes its value as Typst markup, and
`mt add` refuses a value that does not parse as markup. Use `raw` for body
sections that hold lists and code, as the template above does.

= See the slots

`mt add --describe` lists each slot with its kind, whether it is required, and
its prompt:

#terminal(read("../generated/api-service-describe.txt"))

`--describe=json` prints the same as a JSON array, for a script or an agent.

= Fill the slots

#table(
  columns: 2,
  table.header([Source], [Example]),
  [A values file, YAML or JSON], [`mt add -f values.yaml`],
  [Values on stdin], [`mt add -f -`],
  [A token], [`mt add --set src-files=2`, or `mt add -- src-files=2`],
  [Title words], [`mt add Log slow database queries`],
  [An environment variable], [`MINDTAPE_ADD_SRC_FILES=2 mt add`],
  [The slot's `default`], [`slot("src-files", kind: "int", default: 1)`],
)

When two sources set the same slot, the later one wins. `mt add` reads `-f`
files first, left to right, then `--set` tokens, then the tokens after `--`.
Title words, environment variables and defaults fill only the slots still
empty, in that order. A slot reads the variable its `env` names, or else
`MINDTAPE_ADD_` and its name in capitals, with `-` turned into `_`.

A token takes one of three forms:

#table(
  columns: 2,
  table.header([Token], [Effect]),
  [`key=value`], [Sets the slot.],
  [`key+=value`], [Appends to a `list` slot. Repeat it for more items.],
  [`key@path`], [Reads the value from a file, dropping its trailing newlines.],
)

The first `=` or `@` in a token ends the slot name, so `title=a@b` sets the
title `a@b`. A `list` slot refuses `key=value`, because one string is not a
list. A title word that starts with `+`, as in `mt add Fix login +auth`, goes
to the `tags` slot when the template has one. `mt add` refuses a name that no
slot has and lists the template's slots.

= Values from a file

`-f` suits long values. A YAML block scalar (`|`) keeps line breaks and needs
no shell quoting, so a `raw` section reads the same here as in the task. This
is `values.yaml`:

#raw(read("../generated/api-service-values.yaml"), lang: "yaml", block: true)

`mt add -f values.yaml --print` checks the task and prints it without writing
it:

#raw(read("../generated/api-service-add-print.typ"), lang: "typ", block: true)

A `.json`, `.yaml` or `.yml` extension picks the parser. For stdin and any
other name, `mt add` tries JSON first when the text starts with `{` or `[`, and
YAML otherwise. You can pass `-f` more than once. A later file's value replaces
an earlier one, and `null` clears it.

= What `mt add` sets for you

- `status`: the first status in the legend that is not terminal, with
  today's date. `--status <name>` picks another.
- `id`: a new UUID, only when `[tasks.id] format` is `uuid-v4` or `uuid-v7`.
  Otherwise the file name is the task's identity.
- Required fields the template leaves out. When the config requires
  `priority`, `difficulty`, `kind`, `assignee` or `estimate` and the template
  does not set it, `mt add` adds a slot for it. The slot shows up in
  `--describe` and the editor. The `assignee` slot reads the
  `MINDTAPE_PRINCIPAL` environment variable.
  A priority that must come from a framework gets one slot per framework
  input, as #link(u("/priority/"))[Priority] describes.

= Where the file goes

`mt add` writes to the first task root and names the file after the title:
`log-slow-database-queries.typ`. `--name <stem>` sets the name, and
`--to <folder>` files the task in a folder under the root, where it can pick up
#link(u("/derived-tags/"))[derived tags]. `--into <dir>` writes to another
directory. When a file with that name exists, `--dedup` decides: `fail`, `skip`
(write nothing and exit 0) or `add-suffix` (write `<name>-2.typ`).

= Fill the slots in an editor

`mt add --edit` opens your editor on a YAML file with one entry per slot,
filled with the values you passed. `mt add` reads the file back when you save
and close it. In a terminal, `mt add` opens the editor by itself when a
required slot is still empty. `[add] interactive` set to `always` or `never`
changes that. Without a terminal on stdin and stdout, `mt add` opens no
editor, and a required slot left empty fails the add with the slot's name.

The editor is `[add] editor`, else `$VISUAL`, else `$EDITOR`, else `vi`
(`notepad` on Windows). `[add] header` replaces the instructions at the top of
the file, and `[add] comments = false` leaves out every comment.
