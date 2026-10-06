#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": CLI_GROUPS, u
#import "../utils/cli-ref.typ": CLI, cli-gate, cli-group, render-globals

#let meta = (
  route: "/cli/",
  label: "CLI",
  title: "CLI reference",
  summary: "Every `mt` command by group, each linking to its page of flags, notes and generated transcripts.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`mt` is the whole interface. A command prints nothing it does not have to,
and its exit code says whether it did what was asked. The build generates
these pages from the binary: the commands and flags come from `mt`'s own
argument parser (#link(u("/agents/"))[`mt _cli --json`]). Each example is a
command the build ran in a terminal, most of them in the
`examples/api-service` project, a backend team's backlog. The page prints the
command from that run.

#terminal("version")

// The command tree, read from the binary. cli-notes.typ adds prose and
// examples. cli-gate fails the build when a command has no group or a note
// names no command, so a new command cannot go missing from this index.
#cli-gate(CLI, CLI_GROUPS)

= Global flags

Every command accepts the same global flags:

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

= Views

#cli-group(CLI, CLI_GROUPS, "Views")

= Status

#cli-group(CLI, CLI_GROUPS, "Status")

= Recurring tasks

#cli-group(CLI, CLI_GROUPS, "Recurring tasks")

= Validate and inspect config

#cli-group(CLI, CLI_GROUPS, "Validate & inspect config")

= Retire

#cli-group(CLI, CLI_GROUPS, "Retire")

= Setup

#link(u("/quickstart/"))[Quickstart], #link(u("/config/"))[Configuration]
and #link(u("/agents/"))[For AI agents] describe the files these commands
write.

#cli-group(CLI, CLI_GROUPS, "Setup")

#callout("note")[
  #link(u("/agents/"))[For AI agents] collects the JSON output, exit codes
  and environment variables a script or an agent reads.
]
