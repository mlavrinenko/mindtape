#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": REPO, mindtape, u
#import "../utils/release.typ": cli-crate, release-targets

#let meta = (
  route: "/quickstart/",
  label: "Quickstart",
  title: "Quickstart",
  summary: "Install "
    + mindtape
    + ", make a project, and file, list and finish your first tasks.",
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

= Set up the environment

Task files import a Typst prelude, and `mt install` puts it where Typst looks
for it. Run it once per machine:

#terminal("qs-install")

`mt install` also prints a line that turns on Tab completion. Paste it into
your shell's startup file, and Tab completes task names, statuses and
#link(u("/named-sets/#views"))[views] for the project you're in.

Then set up a project, once per repository:

#terminal("qs-init")

Plain `mt init` asks a few questions first. `--no-input` skips them and writes
the default preset, `flow`. `mt init --list-presets` lists the others. The
#link(u("/cli/init/"))[`mt init` page] covers the questions and the presets,
and #link(u("/config/"))[Configuration] covers what the file can hold.

= File your first task

A task is a plain text file, so you can write one in any editor. Or let
`mt add` fill in a template for you. The default template only asks for a
title:

#terminal("qs-add")

Here's the file it wrote:

#raw(read("../generated/qs-written.typ"), lang: "typ", block: true)

#link(u("/task-files/"))[Task files] walks through each line.
#link(u("/templates/"))[Templates & slots] covers editor mode, named templates
and templates that ask for more than a title. Each value a template asks for
is a slot.

= List and query

`mt ls` shows your open tasks. It hides done and cancelled ones, and any task
whose `start` date is still in the future:

#terminal("qs-ls")

Pass a filter expression to narrow the list:

#terminal("qs-filter")

#link(u("/filters/"))[Filters] covers the language: tags, dates, dependency
state and text search. The #link(u("/cli/ls/"))[`mt ls` page] covers the
listing itself: which columns show, the `file` column, hyperlinks and the
formats for other programs. To search by text, use
#link(u("/cli/find/"))[`mt find`].

= Change status

Your project decides which statuses exist and which moves between them are
allowed. It all lives in `.mindtape/config.toml`. Each status is also a
command that sets the status and stamps today's date. The `flow` preset won't
let you jump from proposed straight to done:

#terminal("qs-refused")

Start the task, then finish it:

#terminal("qs-flip")

`mt done x` is short for `mt flip done x`.
#link(u("/statuses/"))[Statuses] covers the moves and the notes a
status can require. The #link(u("/cli/"))[CLI reference] lists every command.

= Validate

`mt check` validates every task against the project's config:

#terminal("qs-check")

It names any violation and exits non-zero, so you can run it from a
pre-commit hook or CI. The #link(u("/cli/check/"))[`mt check` page] lists
what it checks, and #link(u("/rules/"))[Rules] covers the checks you add
yourself.

#callout("note")[
  Curious why a task is a file and how `mt` reads them? Take a look at
  #link(u("/concepts/"))[Concepts].
]
