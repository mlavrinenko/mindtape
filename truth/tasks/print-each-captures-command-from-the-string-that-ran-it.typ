#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print each capture's command from the string that ran it",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("docs", "www"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a newcomer who follows the quickstart gets the output it shows],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[a page cannot show one command and the output of another],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
  links: depends-on(
    "refuse-hand-typed-code-blocks-in-doc-sources.typ",
    "pin-the-docs-capture-environment-and-refuse-machine-paths-in-captures.typ",
  ),
)

== Problem

+ Run the README Usage block in a fresh `mt init` project.
+ 🔴 `mt add --set title="Wire OAuth" --set priority=7` prints `unknown --set/-f key(s): priority; known slots: title`, and `mt flip done wire-oauth` prints `illegal transition proposed → done`.
+ 🔴 /quickstart/ shows `mt add ... --set impact=6 --set ease=5`, which fails the same way after `mt init`: the capture ran in `examples/learn-piano`, and the page does not say so.
+ 🔴 Under "Pass a filter expression" the quickstart shows no filter, and the output equals plain `mt ls`. The command lives in `www/generate.sh` and the page types its own copy, or none.


== TO-BE

- 🟢 Each capture records its command line, its project and its output. `terminal(<slug>)` prints `$ <command>` from that record, then the output.
- 🟢 The quickstart runs in a project `www/generate.sh` creates with `mt init --no-input`, so a reader who follows it gets the same output.
- 🟢 README Usage is removed; README links the quickstart.


== Changes

- `www/generate.sh`: a `capture <slug> <dir> -- <argv>` function writing `<slug>.json` with `cmd`, `project` and `out`; a scratch `mt init --no-input` project for the quickstart. First make `quickstart.typ` read a capture whose command differs from its typed block, and see the render fail.
- `www/utils/ui.typ`: `terminal()` and `terminal-ansi()` take a slug and print the command.
- `www/utils/cli-notes.typ`: blocks through the new helper.
- `www/content/quickstart.typ`, `www/content/cli.typ`: use it; drop their typed command blocks from the block-gate list.
- `docs/src/readme.typ`: drop the Usage section.


== Rollback

Revert the commit; the pages go back to typed command blocks.


== Out of scope

- Other pages' captures: each page task moves its own.


== Dogfood

This repo's quickstart and CLI pages render from the new captures on every `just check`.

