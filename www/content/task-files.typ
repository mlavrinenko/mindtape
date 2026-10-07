#import "../utils/page.typ": page
#import "../utils/ui.typ": pkg, terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/task-files/",
  label: "Task files",
  title: "Read and write a task file",
  summary: "The Typst a task file uses, line by line, and what `mt check` prints for each mistake that syntax invites.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// The file below is examples/task-files/tasks/fix-the-login-redirect.typ,
// which `just check-examples` validates. Each pair joins a slice of that file
// with a slice of a broken file under examples/task-files/invalid/, and each
// terminal is www/generate-task-files.sh's capture of `mt check` refusing the
// broken file.
#let s = checked-slices(
  json("../generated/task-files.json"),
  "task-files",
  ("comma", "note", "dictionary", "array", "links", "hash")
    .map(id => (id, id + "-invalid"))
    .flatten(),
)

#let typst-doc(path, body) = link("https://typst.app/docs/" + path, body)

// One block: the lines that work, then the same lines broken.
#let pair(id) = raw(
  render-slices(
    (works: s.at(id), fails: s.at(id + "-invalid")),
    (("works", "works"), ("fails", "fails")),
  ),
  lang: "typ",
  block: true,
)

A task is one Typst file. This page walks one task file, then shows each
mistake its syntax invites and what `mt check` prints for it. A task file needs
no Typst beyond what this page shows.

#raw(read("../generated/task-files-source.typ"), lang: "typ", block: true)

= The import

The first line imports #pkg, the MindTape prelude. The `: *` makes every name
the prelude defines usable in the file: `task`, the status functions such as
`wip`, and the link functions such as `depends-on`.

`@local` names the Typst packages installed on this machine. `mt install` puts
the prelude there, as the #link(u("/quickstart/"))[Quickstart] shows, so the
import resolves on every machine that ran it.
#link(u("/config/"))[Configuration] shows how a project points the import at a
prelude in its own tree instead.

= The show rule

In Typst markup, `#` starts code, and the code runs to the end of the
expression. `#show: task.with(...)` hands the rest of the file to the function
`task`. `task.with(...)` sets `task`'s named arguments, which are the fields,
and Typst passes the rest of the file as the body. `task` records the fields
for `mt` to read, and lays the task out as a card when Typst compiles the
file.

Each field is `name: value`, and a comma separates it from the next. A value is
a string in double quotes, a number, a function call such as `wip(...)`, an
array or a dictionary. #link(u("/dsl/"))[Task DSL] lists every field and the
values it takes.

= The body

Everything below the show rule is the body, in Typst markup. `=` starts a
heading, `+` a numbered list item, `-` a bullet item, and backticks mark code.
Typst's #typst-doc("reference/syntax/")[syntax reference] lists the rest of
the markup.

= Mistakes and what `mt check` prints

Each block below shows lines from the file above, then the same lines broken.
`mt check` reads every task file, names each one that fails at the line at
fault, and exits 1.
Every other command warns about a file that fails and leaves its task out.

== Separate fields with commas

Without the comma, Typst cannot tell where the title ends:

#pair("comma")

#terminal("task-files-missing-comma")

== Write a dictionary in parentheses

A dictionary is `(key: value, ...)`, in parentheses as an array is. Braces open
a code block, so a dictionary written in braces fails:

#pair("dictionary")

#terminal("task-files-dictionary-in-braces")

== End a one-item array with a comma

Parentheses around one value only group it, so `("backend")` is the string
`"backend"`. A trailing comma makes it an array of one item:

#pair("array")

#terminal("task-files-one-tag-without-a-comma")

== Write a note in square brackets

A status function takes the year, month and day, then an optional note in
square brackets after the call. Square brackets hold content, which is Typst
markup. A note passed as a string fails:

#pair("note")

#terminal("task-files-note-as-a-string")

== Join links with `+`

Each link function returns an array of links, and `+` joins two arrays into
one. A comma ends the `links` field instead, so the second call becomes an
argument of `task` with no name:

#pair("links")

#terminal("task-files-links-joined-with-a-comma")

== Escape `#` in the body

The body is markup, where `#` starts code. So `#backend` is code, and Typst
looks up a variable named `backend`:

#pair("hash")

#terminal("task-files-hash-in-the-body")

= The rest of Typst

A task file uses only the syntax above. The Typst documentation covers the
rest: #typst-doc("reference/scripting/")[scripting] for variables, functions
and imports, #typst-doc("reference/foundations/array/")[arrays],
#typst-doc("reference/foundations/dictionary/")[dictionaries],
#typst-doc("reference/foundations/content/")[content] and
#typst-doc("reference/styling/")[show rules].
