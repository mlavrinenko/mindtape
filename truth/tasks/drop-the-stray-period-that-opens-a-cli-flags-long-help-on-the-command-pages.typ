#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop the stray period that opens a CLI flag's long help on the command pages",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "www"),
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
      docs-match-code: helps[the CLI pages show clap's long help as clap prints it],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 6),
)

== Problem

+ Rendering the CLI pages to Markdown for llms.txt
+ 🔴 #raw("www/utils/cli-ref.typ"): \`\_desc\` strips the short help from the front of the long help, and when clap's long help repeats it with a trailing period, the period stays behind as a paragraph of its own

```
$ rg -o 'each legend value is a verb.{0,40}' www/public/mindtape/cli/flip/index.html
each legend value is a verb</p><p>.<br /><br />Stamps `&lt;status&gt;` dated
```
11 commands and arguments in `www/generated/cli.json` have a long help that is the short help plus a period.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("www/utils/cli-ref.typ").

== TO-BE

- 🟢 `/cli/flip/` shows the short help, then `Stamps <status> dated today…` as the next paragraph, with no paragraph holding a lone `.`.


== Changes

- `www/utils/cli-ref.typ`: in `_desc`, also strip a leading `.` and the blank lines after it from the long-help remainder.


== Rollback

- Revert the `_desc` change.


== Out of scope

- Rendering the backticks in clap's help as code spans. The pages show them as typed.


== Dogfood

Every command page this repo's docs site serves, and its Markdown copy, read without the stray period.

