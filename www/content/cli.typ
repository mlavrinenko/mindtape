#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": CLI_GROUPS, u
#import "../utils/cli-ref.typ": CLI, cli-gate, cli-group, render-globals

#let meta = (
  route: "/cli/",
  label: "CLI reference",
  title: "CLI reference",
  summary: "Every `mt` command by group, each linking to its page of flags, notes and generated transcripts.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

Your task files are plain text, and you can edit them in any editor. `mt`
does the fiddly parts for you: it stamps status dates, writes link paths,
fills template slots and checks every task. Each command prints only what you
need, and its exit code tells you whether it did what you asked.

The build generates these pages from the binary: the commands and flags come
from `mt`'s own argument parser (#link(u("/agents/"))[`mt _cli --json`]).
Each example is a command the build ran in a terminal, most of them in
`examples/api-service`, a sample backend project's backlog, so what you see is
what that run printed.

#terminal("version")

// The command tree, read from the binary. cli-notes.typ adds prose and
// examples. cli-gate fails the build when a command has no group or a note
// names no command, so a new command cannot go missing from this index.
#cli-gate(CLI, CLI_GROUPS)

= Global flags

Every command takes these flags:

#render-globals(CLI)

= Files

#cli-group(CLI, CLI_GROUPS, "Files")

= Fields

#cli-group(CLI, CLI_GROUPS, "Fields")

= Listing

#cli-group(CLI, CLI_GROUPS, "Listing")

= Graph

#cli-group(CLI, CLI_GROUPS, "Graph")

= Links

#cli-group(CLI, CLI_GROUPS, "Links")

= Views and dictionaries

#cli-group(CLI, CLI_GROUPS, "Views & dictionaries")

= Status

#cli-group(CLI, CLI_GROUPS, "Status")

= Recurring tasks

#cli-group(CLI, CLI_GROUPS, "Recurring tasks")

= Validate and inspect config

#cli-group(CLI, CLI_GROUPS, "Validate & inspect config")

= Retire

#cli-group(CLI, CLI_GROUPS, "Retire")

= Setup

These commands write the files that #link(u("/quickstart/"))[Quickstart],
#link(u("/config/"))[Configuration] and #link(u("/agents/"))[For AI agents]
describe.

#cli-group(CLI, CLI_GROUPS, "Setup")

#callout("note")[
  If a script or an agent drives `mt`, #link(u("/agents/"))[For AI agents]
  collects the JSON output, exit codes and environment variables it reads.
]
