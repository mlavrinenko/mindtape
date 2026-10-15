#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Make the split-oversized-tasks page quote the refusal's exit code",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
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
      docs-match-code: helps[the page states the exit code the refused mt add returned],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 16),
)

== Problem

+ Writing the verb-first titles recipe, which reuses `exit-code()` from `www/utils/ui.typ`.
+ 🔴 #raw("www/content/split-oversized-tasks.typ"): `exit-code(slug)` returns the last step's code, and the split-oversized-tasks session ends with the rename that files, so the page says the refused `mt add` exits 0.

```
$ jq '[.steps[].code]' www/generated/split-oversized-tasks.json
[1, 0]
$ grep -n 'exits #exit-code' www/content/split-oversized-tasks.typ
The refused `mt add` exits #exit-code("split-oversized-tasks") and writes no
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("www/utils/ui.typ").

== TO-BE

- 🟢 The split-oversized-tasks page says the refused `mt add` exits 1.
- 🟢 `exit-code()` can't quote a passing step's code where a page means the refusal.


== Changes

- `www/utils/ui.typ`: read the first failing step's code, or take the step to read.
- `www/content/split-oversized-tasks.typ`: quote the refusal's code.


== Rollback

- Revert both files.


== Out of scope

- Other recipe prose. Each page's polish is its own task.


== Dogfood

Not applicable. The change is docs.
