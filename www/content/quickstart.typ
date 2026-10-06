#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, pkg, terminal
#import "../utils/site.typ": REPO, u
#import "../utils/release.typ": cli-crate, release-targets

#let meta = (
  route: "/quickstart/",
  label: "Quickstart",
  title: "Quickstart",
  summary: "Install MindTape, make a project, and file, list and finish your first tasks.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// Every transcript below comes from one session that www/generate-quickstart.sh
// runs in an empty directory, in the order this page shows it.

= Install

Install the CLI crate from crates.io:

// docs:hand-typed the crate name comes from the release workflow, and the build does not run cargo install
#raw("cargo install " + cli-crate, lang: "sh", block: true)

Each tagged release also builds a binary for #release-targets.len() platforms.
Download the one for yours from the #link(REPO + "/releases")[Releases page]:

#list(..release-targets.map(t => raw(t.artifact)))

Or run it from the flake without installing it:

// docs:hand-typed the build does not fetch the flake from GitHub
```sh
nix run github:mlavrinenko/mindtape -- --help
```

Or build it from source with a Rust toolchain. The binary is
`target/release/mt`:

// docs:hand-typed the build does not clone the repository it runs in
```sh
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape && cargo build --release
```

These builds leave out the `rrule` cargo feature. Add it only if you need
calendar rules such as the first Monday of each month, as
#link(u("/recurring/"))[Recurring tasks] explains.

= Set up the environment

Task files import a Typst prelude, and `mt install` puts it where Typst looks.
Run it once per machine:

#terminal("qs-install")

Every task file opens by importing #pkg, and Typst finds it in that
machine-wide `@local` directory. #link(u("/config/"))[Configuration] shows how
to pin a project to a prelude in its own tree.

`mt install` also writes a completion hook for each shell. Source the one for
your shell from its startup file. Tab then completes the task stems (each task
file's name without `.typ`), statuses, #link(u("/named-sets/#views"))[views],
templates and link kinds of the project you are in. The hook asks `mt` on every
Tab press, so its answers always match the project.

Then make a project, once per repository:

#terminal("qs-init")

Plain `mt init` asks a few questions first. `--no-input` writes the default
preset, `flow`, and `mt init --list-presets` lists the others. The
#link(u("/cli/init/"))[`mt init` page] covers the questions and the presets.
#link(u("/config/"))[Configuration] covers what the written file can hold.

= File your first task

`mt add` files a task from a template. Each value the template asks for is a
slot, and the default template has one slot, the title, so a title is all it
needs:

#terminal("qs-add")

The written file is plain Typst that holds the values you gave:

#raw(read("../generated/qs-written.typ"), lang: "typ", block: true)

#link(u("/task-files/"))[Task files] explains each line of it.

#link(u("/templates/"))[Templates & slots] covers templates with more slots,
editor mode and named templates.

= List and query

`mt ls` lists the tasks in the default view, the view a bare `mt ls` applies.
In a new project those are the tasks that are not done or cancelled and whose
`start` date, if they have one, has come:

#terminal("qs-ls")

A filter expression narrows the list:

#terminal("qs-filter")

#link(u("/filters/"))[Filters] covers the language: tags, dates, dependency
state and text search. The #link(u("/cli/ls/"))[`mt ls` page] covers the listing
itself: which columns show, the `file` column, hyperlinks and the formats for
other programs. #link(u("/cli/find/"))[`mt find`] searches by text.

= Change status

The project's legend lists the statuses a task may hold, in order. Each status
in it is also a command, and it stamps today's date. The legend also lists the
moves each status allows. `flow` allows
`proposed` to `wip` and `wip` to `done`, so `mt` refuses a jump straight to
`done`:

#terminal("qs-refused")

Start the task, then finish it:

#terminal("qs-flip")

`mt done x` is short for `mt flip done x`.
#link(u("/statuses/"))[The status legend] covers the moves and the notes a
status can require, and the #link(u("/cli/"))[CLI reference] lists every
command.

= Validate

`mt check` validates every task against the project's config:

#terminal("qs-check")

It exits non-zero on a violation and names it, so it fits a pre-commit hook or
CI. The #link(u("/cli/check/"))[`mt check` page] lists what it checks, and
#link(u("/rules/"))[Rules] covers the checks a project adds.

#callout("note")[
  #link(u("/concepts/"))[Concepts] explains the model: why a task is a file, and
  what live evaluation means.
]
