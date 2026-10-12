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

Each tagged release also ships a binary for #release-targets.len() platforms.
Grab yours from the #link(REPO + "/releases")[Releases page]:

#list(..release-targets.map(t => raw(t.artifact)))

Or run it from the flake without installing:

// docs:hand-typed the build does not fetch the flake from GitHub
```sh
nix run github:mlavrinenko/mindtape -- --help
```

Or build it from source with a Rust toolchain. The binary lands in
`target/release/mt`:

// docs:hand-typed the build does not clone the repository it runs in
```sh
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape && cargo build --release
```

These builds leave out the `rrule` cargo feature. You only need it for
calendar rules like "the first Monday of each month". See
#link(u("/recurring/"))[Recurring tasks].

= Set up the environment

Task files import a Typst prelude, and `mt install` puts it where Typst looks
for it. Run it once per machine:

#terminal("qs-install")

Every task file starts by importing #pkg, and Typst finds it in that
machine-wide `@local` directory. To pin a project to a prelude in its own tree,
see #link(u("/config/"))[Configuration].

`mt install` also writes a completion hook for each shell. Source the one for
your shell from its startup file, and Tab completes the task stems (each task
file's name without `.typ`), statuses, #link(u("/named-sets/#views"))[views],
templates and link kinds of the project you're in. The hook asks `mt` on every
Tab press, so the answers always match the project.

Then set up a project, once per repository:

#terminal("qs-init")

Plain `mt init` asks a few questions first. `--no-input` skips them and writes
the default preset, `flow`. `mt init --list-presets` lists the others. The
#link(u("/cli/init/"))[`mt init` page] covers the questions and the presets,
and #link(u("/config/"))[Configuration] covers what the file can hold.

= File your first task

A task is a plain text file, so you can write one in any editor. `mt add` is
the quick way: it fills in a template. Each value the template asks for is a
slot. The default template has one slot, the title, so a title is all you
pass:

#terminal("qs-add")

Here's the file it wrote:

#raw(read("../generated/qs-written.typ"), lang: "typ", block: true)

#link(u("/task-files/"))[Task files] walks through each line.
#link(u("/templates/"))[Templates & slots] covers templates with more slots,
editor mode and named templates.

= List and query

`mt ls` lists the tasks in the default view, which is what a bare `mt ls`
shows. In a new project that's every task that isn't done or cancelled, and
whose `start` date, if it has one, has come:

#terminal("qs-ls")

Pass a filter expression to narrow the list:

#terminal("qs-filter")

#link(u("/filters/"))[Filters] covers the language: tags, dates, dependency
state and text search. The #link(u("/cli/ls/"))[`mt ls` page] covers the
listing itself: which columns show, the `file` column, hyperlinks and the
formats for other programs. To search by text, use
#link(u("/cli/find/"))[`mt find`].

= Change status

The project's legend lists the statuses a task can have, in order, and the
moves each status allows. Both are yours to change in `.mindtape/config.toml`.
Each status is also a command that sets the status and stamps today's date.
The `flow` preset allows `proposed` to `wip` and `wip` to `done`, so `mt`
won't jump straight to `done`:

#terminal("qs-refused")

Start the task, then finish it:

#terminal("qs-flip")

`mt done x` is short for `mt flip done x`.
#link(u("/statuses/"))[The status legend] covers the moves and the notes a
status can require. The #link(u("/cli/"))[CLI reference] lists every command.

= Validate

`mt check` validates every task against the project's config:

#terminal("qs-check")

It names any violation and exits non-zero, so you can run it from a
pre-commit hook or CI. The #link(u("/cli/check/"))[`mt check` page] lists
what it checks, and #link(u("/rules/"))[Rules] covers the checks you add
yourself.

#callout("note")[
  #link(u("/concepts/"))[Concepts] explains the model: why a task is a file, and
  what live evaluation means.
]
