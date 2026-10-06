#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Give every CLI command its own docs page",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("www", "docs"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a reader looking up one command reads that command, not a 7,400-word page],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[a command with no page fails the docs gate, as a command with no note does now],
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 6),
)

== Problem

+ Open `/cli/` on the docs site and look for `mt link`.
+ 🔴 One page of about 7,400 words holds all the commands. The reader scrolls past twenty others, and a search hit or an agent fetching the page gets all of them.


== TO-BE

- 🟢 `/cli/` is an index: the commands in their current groups, each with its one-line summary, linking to `/cli/<command>/`.
- 🟢 `/cli/<command>/` holds that command's usage, arguments, flags, notes and transcripts, and a subcommand such as `mt config show` sits on its parent's page.
- 🟢 The sidebar keeps one CLI entry. Search and the index reach the command pages.
- 🟢 A command with no page fails the docs gate, and every link that pointed into a `/cli/` anchor points to the command's page.


== Source of truth

- `mt _cli --json`, captured as `www/generated/cli.json`: the commands, flags and summaries.
- `www/utils/cli-notes.typ`: the per-command prose and examples.


== Changes

- `www/utils/cli-ref.typ`: render one command's page and the index from `cli.json`.
- `www/content/cli.typ`: the index.
- `www/content/cli/<command>.typ`, new, one per command: a stub calling the renderer. If tola can build a page per data entry, use that instead of stubs.
- The script behind `check-doc-coverage`: require a page for every command.
- `www/content/*.typ` and `docs/src/*.typ`: point `/cli/` anchor links at the command pages.


== Out of scope

- Rewriting the command notes or the flag help, which comes from clap.
- The sidebar groups, which another task regroups.


== Dogfood

An agent in this repo fetches the one command page it needs instead of the whole reference.

