#import "../utils/page.typ": page
#import "../utils/ui.typ": pkg, terminal
#import "../utils/site.typ": mindtape, u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/task-files/",
  label: "Task files",
  title: "Task files",
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

A task is one Typst file. You can write it in any editor, and you don't need
any Typst beyond what's on this page. Here's a whole task file. Below it, each
part is explained, then each mistake the syntax invites and what `mt check`
prints for it.

#raw(read("../generated/task-files-source.typ"), lang: "typ", block: true)

= The import

The first line imports #pkg, the #mindtape prelude. The `: *` brings in every
name the prelude defines: `task`, the status functions such as `wip`, and the
link functions such as `depends-on`.

`@local` means the Typst packages installed on this machine. Where none holds
the prelude, `mt` reads the copy built into it, so `mt` itself works anywhere.
`mt install` puts the prelude there for your editor and `typst`, as the
#link(u("/quickstart/"))[Quickstart] shows. To point the import at a prelude in
the project's own tree instead, see #link(u("/config/"))[Configuration].

= The show rule

In Typst markup, `#` starts code, and the code runs to the end of the
expression. `#show: task.with(...)` hands the rest of the file to the function
`task`. `task.with(...)` sets `task`'s named arguments, which are the fields,
and Typst passes the rest of the file in as the body. `task` records the
fields for `mt` to read, and lays the task out as a card when Typst compiles
the file.

Each field is `name: value`, with a comma before the next one. A value is a
string in double quotes, a number, a function call such as `wip(...)`, an
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
fault, and exits 1. Every other command warns about a file that fails and
leaves its task out.

== Separate fields with commas

Without the comma, Typst can't tell where the title ends:

#pair("comma")

#terminal("task-files-missing-comma")

== Write a dictionary in parentheses

A dictionary is `(key: value, ...)`, in parentheses like an array. Braces open
a code block, so a dictionary in braces fails:

#pair("dictionary")

#terminal("task-files-dictionary-in-braces")

== End a one-item array with a comma

Parentheses around one value only group it, so `("backend")` is the string
`"backend"`. Add a trailing comma to make it an array of one item:

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
one. A comma ends the `links` field instead, so the second call turns into an
unnamed argument of `task`:

#pair("links")

#terminal("task-files-links-joined-with-a-comma")

== Escape `#` in the body

The body is markup, where `#` starts code. So `#backend` is code, and Typst
looks for a variable named `backend`:

#pair("hash")

#terminal("task-files-hash-in-the-body")

= The rest of Typst

A task file only needs the syntax above. For the rest, see the Typst docs:
#typst-doc("reference/scripting/")[scripting] for variables, functions and
imports, #typst-doc("reference/foundations/array/")[arrays],
#typst-doc("reference/foundations/dictionary/")[dictionaries],
#typst-doc("reference/foundations/content/")[content] and
#typst-doc("reference/styling/")[show rules].
