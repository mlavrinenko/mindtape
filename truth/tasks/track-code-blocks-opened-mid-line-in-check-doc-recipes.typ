#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Track code blocks opened mid-line in check-doc-recipes",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[a hand-typed recipe name in cli-notes.typ prose fails the gate again],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== Problem

+ writing check-doc-spans.sh, which scans the same doc sources
+ 🔴 #raw(".just/scripts/check-doc-recipes.sh"): The awk tagger toggles the block state only on a line that starts with three backticks, so a block opened mid-line leaves the state wrong after it.

`www/utils/cli-notes.typ` opens 18 blocks as #raw("examples: ```sh"). The tagger reads the shell lines inside each such block as prose, and the prose after every second one as block content. There it checks only a leading `just <name>`, so a backtick `just <name>` span passes unchecked.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/check-doc-recipes.sh").

== TO-BE

- 🟢 Every three-backtick run toggles the block state wherever it sits on the line, as `spans()` in `.just/scripts/check-doc-spans.sh` does.
- 🟢 A control proves a block opened mid-line hides its content and leaves the prose after it checked.


== Changes

- `.just/scripts/check-doc-recipes.sh`: split each line on three backticks and toggle per delimiter; add the control case.


== Rollback

Revert the commit.

== Out of scope

- Merging the two scanners into one helper: worth weighing once both read blocks the same way.


== Dogfood

`just check` runs the gate on every commit, so cli-notes.typ prose is checked on the next one.
