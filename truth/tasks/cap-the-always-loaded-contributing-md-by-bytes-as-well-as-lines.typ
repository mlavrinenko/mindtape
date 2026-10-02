#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Cap the always-loaded CONTRIBUTING.md by bytes as well as lines",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 1),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent session pays a bounded, known cost for CONTRIBUTING.md],
      enforced-practice: neutral,
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
  status: proposed(2026, 10, 3),
)

== Problem

+ Setting the linecop cap on CONTRIBUTING.md while cutting it to the rules
+ 🔴 #raw(".linecop.yaml"): linecop counts lines only, and typlite writes one line per paragraph, so the 142-line cap on CONTRIBUTING.md passes the 30 KB version that the cut removed.

```
$ git show e3a52225:CONTRIBUTING.md | wc -lc
  128 30853
$ linecop --format paths   # that file restored, `./CONTRIBUTING.md` at `limit: 142`
$ echo $?
0
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".linecop.yaml").

== TO-BE

- 🟢 A gate fails when CONTRIBUTING.md grows past a byte budget, so a paragraph that grows into an essay fails like a new paragraph does.


== Changes

- linecop (mlavrinenko/linecop): a byte limit per override, beside the line `limit`.
- `.linecop.yaml`: set the byte limit on `./CONTRIBUTING.md` at its size plus about 10%.


== Rollback

Remove the byte limit from `.linecop.yaml`. The line cap stays.

== Out of scope

- The caps on other files, where a line count measures size well.


== Dogfood

Every agent session in this repo loads CONTRIBUTING.md whole, so its byte budget bounds what each session spends on it.
