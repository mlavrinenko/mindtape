#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Remove the PENDING list from the check-doc-prose gate",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 4),
  tags: ("found", "gating"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a new violation fails the gate with no exemption list],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 5),
)

== Problem

+ Rewriting the last pending comment files for check-doc-prose
+ 🔴 #raw(".just/scripts/check-doc-prose.sh"): check-doc-prose.pending is empty, yet the script still reads it and runs a pending\_filter arm with its own control

```
$ wc -c .just/scripts/check-doc-prose.pending
0 .just/scripts/check-doc-prose.pending
$ grep -c -i pending .just/scripts/check-doc-prose.sh
11
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/check-doc-prose.sh").

== TO-BE

- 🟢 A new violation in any scanned file fails `just check-doc-prose` outright, with no list to join.
```sh
$ just check-doc-prose   # a new hit
check-doc-prose: f.sh:3: all-caps "OUT"
```


== Changes

- `.just/scripts/check-doc-prose.pending`: delete.
- `.just/scripts/check-doc-prose.sh`: drop `pending_filter`, its control and the header mention.
- `docs/src/contributing/documentation.typ`: drop the pending sentences; regenerate the Markdown.


== Rollback

- Revert the commit.


== Out of scope

- The PENDING list in `check-doc-blocks`, a separate gate.


== Dogfood

This repo's `just check` runs the gate with no pending list.
