#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/derived-tags/",
  label: "Derived tags",
  title: "Derived tags",
  summary: "How `mt` tags each task with its root and the folders it sits in, without writing the tags to the file.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

If your folders already say what a task is about, you don't have to repeat it
in tags. `mt` can tag each task by where its file sits: the name of its
#link(u("/concepts/#a-task-is-a-file"))[task root] and the folders between the
root and the file. File a task under `api/tasks/auth/` and it matches
`has_tag("auth")` with no tag written in it.

The examples below come from `examples/derived-tags/`, a repo with one task
root per service.

= Turn it on

Derived tags are off by default. Set `enabled = true` under
`[tasks.derive-tags]` to turn them on for every root:

#raw(
  render-slices(
    checked-slices(
      json("../generated/derived-tags-config.json"),
      "derived-tags-config",
      ("roots",),
    ),
    (("roots", "two roots, both deriving tags"),),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

To turn them on for one root, set `derive-tags` on that root's
`[[tasks.roots]]` entry instead. It takes `true`, `false`, a glob or a list of
globs. A glob limits derivation to the files whose path under the root matches
it. A root without its own `derive-tags` follows `enabled`.

= What each task gets

These are the tags `mt` derives for each task in the example:

#let listed = (
  json("../generated/derived-tags-ls.json").tasks.sorted(key: t => t.file)
)
#table(
  columns: 2,
  table.header([File], [Derived tags]),
  ..listed.map(t => (raw(t.file), t.derived_tags.map(raw).join[, ])).flatten(),
)

Every task gets its root's tag: the root's `name`, or the last folder of its
`path` if it has no name. `collect` picks what the folders add:

- `segments`, the default, adds each folder as a tag: `auth`, `sessions`.
- `breadcrumbs` adds the path to each folder, joined with `separator` (`:` by
  default): `auth`, `auth:sessions`. With `breadcrumb-root = true`, each
  breadcrumb starts with the root's tag: `api:auth`.

`mt` lowercases folder names and turns spaces and punctuation into `-`, so a
folder named `Auth Flow` gives `auth-flow`. Set `slugify = false` to keep
names as they are.

= Filter on them

A filter sees derived tags as tags:

#terminal("derived-tags-ls-auth")

`mt ls --json` lists them under `derived_tags`, separate from the tags written
in the file. #link(u("/filters/"))[Filters] covers `has_tag` and the rest of
the language.

= How they behave

`mt` derives the tags each time it loads the tasks and never writes them to
the file. Move a file to another folder and its tags change with it.

If `[tasks.tags] allowed` lists the permitted tags, `mt check` refuses an open
task whose folders give it a tag that isn't on the list. Closed tasks aren't
checked, so you can drop a tag from the list without editing every closed task
that has it. That's `values-when = "!terminal"`, the default for tags. Set
your own filter there to change which tasks the list applies to.
