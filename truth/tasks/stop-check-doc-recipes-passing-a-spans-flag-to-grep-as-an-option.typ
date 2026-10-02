#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Stop check-doc-recipes passing a span's flag to grep as an option",
  kind: "dev",
  priority: ice(
    impact: 2,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
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
      sound-codebase: helps[a gate failure prints only its own message],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== Problem

+ Writing just's --fmt flag in a doc code span while rewriting docs/src/contributing/gates.typ
+ 🔴 #raw(".just/scripts/check-doc-recipes.sh"): A code span such as \`just --fmt\` makes \`is\_recipe\` run \`grep -qxF --fmt\`, so grep prints a usage error and the gate suggests calling \`\#just.--fmt(...)\`.

```
check-doc-recipes: docs/src/contributing/gates.typ:125: literal `just --fmt` outside the generated module — use #just.--fmt(...)
grep: unrecognized option '--fmt'
Usage: grep [OPTION]... PATTERNS [FILE]...
Try 'grep --help' for more information.
check-doc-recipes: docs/src/contributing/gates.typ:125: `just --fmt` is not a real recipe
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/check-doc-recipes.sh").

== TO-BE

- 🟢 A span that names a just flag, such as `just --fmt`, gets one message that says it is a flag and not a recipe, with no grep error.


== Changes

- `.just/scripts/check-doc-recipes.sh`: `is_recipe` passes `--` before the name, and a name that starts with `-` gets its own message.


== Rollback

Revert the commit. The gate still fails on the span, with the grep noise.

== Out of scope

- Which span shapes the gate scans.


== Dogfood

The doc sources here mention just's own flags, and each mention fails with one clear line.
