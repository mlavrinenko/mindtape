#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Silence tola's progress bar in the docs build recipe",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "www"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a quiet build leaves its failures readable],
      ),
      "readable-tasks",
      "implementable-tasks",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (:),
      "sound-codebase",
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 12),
)

== AS-IS

+ Building the docs site to check the comparison matrix fix
+ 🔴 #raw(".just/docs.just"): `just docs build` prints tola's `[build]` progress bar as one carriage-return redraw per page on every clean run.

```
[build] typst(0/54) assets(0/5)␍[build] typst(0/54) assets(1/5)␍ … [build] typst(54/54) assets(5/5)
[sitemap] sitemap.xml
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/docs.just").

== TO-BE

- 🟢 A clean `just docs build` prints nothing from tola, and a failing one prints tola's whole report.


== Changes

- `.just/docs.just`: capture `tola build`'s output in `build` and replay it only on failure, as `check` already does.


== Rollback

- Revert the `build` recipe line.


== Out of scope

- `docs serve`, whose live output is the point of running it.
- A quiet flag upstream in tola, which 0.7.1 lacks.


== Dogfood

Every `just docs build` in this repo prints no progress redraws.
