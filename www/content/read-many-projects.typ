#import "../utils/config-ref.typ": config-keys-table
#import "../utils/page.typ": page
#import "../utils/shipped-board.typ": shipped-board
#import "../utils/set-command-notes.typ": set-command-list
#import "../utils/ui.typ": exit-code, terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/read-many-projects/",
  label: "Read many projects",
  title: "Read many projects",
  summary: "Name your projects in `at.toml`, then see, list and check them all from one `mt @name`.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// The blocks are slices of examples/across-projects/.config/mindtape/at.toml,
// and www/generate-reference.sh runs every command here with that copy as
// `$HOME`, over its two projects, shop and blog, and a fork of shop.
#let slices = checked-slices(
  json("../generated/across-projects-config.json"),
  "across-projects-config",
  ("sets", "forks"),
)
#let at-schema = json("../generated/at-schema.json")

You keep a backlog in every repo, and `mt` reads the one you're standing in.
That's fine until you want to know where you left off across all of them.
Name your projects once, and a leading `@name` reads every one of them.

= Name your projects

`at.toml` sits in your #link(u("/config/"))[config directory], next to
`defaults.toml`:

#raw(
  render-slices(
    slices,
    (("sets", "a project and a set of projects"),),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

`mt install` writes `at.schema.json` beside it. Start `at.toml` with a
`#:schema ./at.schema.json` line, and an editor with tombi or Taplo completes
its keys and checks their values as you type.

Each `[at.<name>]` entry takes these keys:

#config-keys-table(at-schema, "at")

A `*` matches inside one directory level, and a glob keeps only the
directories that hold `.mindtape/`. An entry whose `include` is one literal
path is a project, so `mt @shop ls` is `mt -w ~/projects/shop ls`. Any other
entry is a set, even when it matches one project.

`mt at` lists the entries, and `mt at <name>` lists a set's projects. Each
project goes by its directory name:

#terminal("across-at")

A name with no entry finds the directory of that name in any `include`, so
`mt @blog ls` works with no `[at.blog]`. Got two directories called `blog`?
Each goes by the shortest path suffix that tells them apart, like `oss/blog`,
and `mt @oss/blog ls` works too. A loop of `@name`s that include each other
gets refused, and `mt` prints the loop.

= See every project at once

A bare `mt @work` runs #link(u("/cli/board/"))[`mt board`] on the set, one row
per project:

#terminal("across-board")

Every cell runs under its project's own config. shop's statuses are `todo`,
`wip` and `done`, blog's are `draft`, `in-review` and `done`, and the board
counts both right, because its columns read `!terminal`, `ready` and
`blocked`, never a status name. The board page covers the columns, the hidden
rows and the `⚠` row of a project whose config won't load.

This is the board `mt` ships. It lists the projects with the most ready tasks
first, then the most open ones, and drops a project with nothing open, so a
project with no open task is missing from the table:

#shipped-board

`[board]` in `at.toml` changes the board for every entry, and
`[at.<name>.board]` changes it for one, key by key over the shipped board:

#config-keys-table(at-schema, "board")

Each `[[board.columns]]` entry takes these keys:

#config-keys-table(at-schema, "board.columns")

= List tasks across projects

`mt @work ls` and `mt @work find` print one table, with a `PROJECT` column
first and the rows grouped by project in set order:

#terminal("across-ls")

Each project brings its own default listing, sort and columns, and the table
shows every column any of them shows. `--sort` sorts inside each project, and
`-n` and `--skip` count per project, so `-n 1` gives you each project's top
task. `--format csv` adds a `project` column. A project that fails to list
prints its error after the others, and `mt` exits 1. For a script,
`--json` has the #link(u("/schema/#many-projects"))[keys a set adds].

= Check every project

`mt @work check` checks each project under its own rules, prints every
problem with the project it belongs to, and ends with one line:

#terminal("across-check")

It exits 1 when any project fails, a config that won't parse included, so a
nightly job can watch all your repos at once. `--json` prints one object per
project. `--commit-msg` and `--commits` check one repository's commits, so
they refuse a set.

= Write in one project

Only #set-command-list() run on a set. Every other command, and every write,
takes one project and refuses a set with exit code
#exit-code("across-write"):

#terminal("across-write")

= Forks

A fork is a linked git worktree, the kind `git worktree add` makes. It holds a
copy of its checkout's tasks, so a set leaves it out unless `forks` says
otherwise:

#raw(
  render-slices(
    slices,
    (("forks", "forks beside their checkouts, and forks alone"),),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

`forks` filters only what a glob matches. A literal path or an `@name` in `include` joins as written.
`mt at` marks each fork. A fork that `@work` leaves out still answers to its
directory name:

#terminal("across-forks")

`-w <path>` and `MINDTAPE_WORKTREE` always name one project, fork or not.
