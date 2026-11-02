#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name tinymist in the Quickstart as the editor that completes task fields",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "www"),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[a newcomer gets the autocompletion the README promises without hunting for how],
      ),
      "readable-tasks",
      "implementable-tasks",
      "enforced-practice",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (
        docs-match-code: helps[the Quickstart sets up what the README claims],
      ),
      "sound-codebase",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 19),
)

== AS-IS

+ Following the Quickstart to the point where the README's autocompletion claim should start to hold
+ 🔴 #raw("www/content/quickstart.typ"): The README promises that the editor completes field names and flags a misspelled one, but the Quickstart names no language server and never links the page that explains `@local` packages

```
$ rg -n 'tinymist' www/content/quickstart.typ; echo "exit=$?"
exit=1
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("www/content/quickstart.typ").

== TO-BE

- 🟢 Right after the `mt install` step, the Quickstart says that tinymist, the Typst language server, finds the installed prelude and completes and checks the `task` fields, and links tinymist and the Task files page.


== Changes

- `www/content/quickstart.typ`: one or two sentences after the `mt install` paragraph, with the links.


== Rollback

- Revert the file.


== Out of scope

- Per-editor setup steps, which tinymist's own docs cover.


== Dogfood

Not applicable, this repo's contributors already run tinymist.
