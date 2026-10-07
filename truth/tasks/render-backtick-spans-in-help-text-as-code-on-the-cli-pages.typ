#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Render backtick spans in help text as code on the CLI pages",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "www"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[the CLI reference pages read as formatted docs, not raw help strings],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 7),
)

== Problem

+ Rewriting the mt help text
+ 🔴 #raw("www/utils/cli-ref.typ"): www/utils/cli-ref.typ renders a command's help as plain text, so a backtick span such as \`-n 20\` prints with its backticks on the /cli/ pages

```
$ mt ls --help
      --skip <SKIP>
          Skip the first N rows after sorting. For page two of `-n 20`, use `--skip 20`. ...
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("www/utils/cli-ref.typ").

== TO-BE

- 🟢 On the /cli/ pages, a backtick span in help text renders as inline code.

== Changes

- `www/utils/cli-ref.typ`: split help strings at backtick pairs and render each span with `raw`.

== Rollback

- Revert `www/utils/cli-ref.typ`.

== Out of scope

- Terminal `--help`, which shows the backticks as typed.

== Dogfood

This repo's site shows every flag's help formatted.
