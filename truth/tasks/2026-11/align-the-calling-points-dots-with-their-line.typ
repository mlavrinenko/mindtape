#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Align the calling points dots with their line",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 10,
  ),
  difficulty: 1,
  tags: ("www", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  status: done(2026, 11, 25),
  assignee: "agent",
)

== AS-IS

+ Run `just docs build`, serve `www/public/` and open `/mindtape/concepts/`.
+ At 390 by 844, open Calling points. The rings sit 1px right of the line, and a green square corner shows behind each ring.
+ At 1360 by 900, the rail rings sit 1px left of the line.
+ 🔴 The same component draws its dots off centre at both widths.


== Environment

- The docs site built from `docs-redesign` at the commit that filed this, in headless Chromium at 390 by 844 and 1360 by 900, on Linux. The site has no browser test harness, so the check ran in the browser.


== TO-BE

- 🟢 Each ring sits on the vertical line in the rail and in the folded list.
- 🟢 The folded list draws only its rings, and no prose square marker sits behind them.
- 🟢 The current stop keeps its paper ring and green fill.

```html
<ul class="page-toc-list">
  <li class="is-current"><a href="#a-task-is-a-file">A task is a file</a></li>
  <li><a href="#every-command-reads-the-files">Every command reads the files</a></li>
</ul>
```


== Changes

- `www/assets/styles/components.css`: move the rail ring from `-5px` to `-4px` and the folded-list ring from `-18px` to `-20px`. A note explains the rail border that shifts the containing box.
- `www/templates/layout.typ`: give the generated list a `page-toc-list` class. The prose bullet rule then leaves the component alone.


== Out of scope

- The line map's stops, which keep their own offsets. This task changes only the calling-points component.
- The scroll tracking, the folded summary and the list's font size. All stay as they are.


== Dogfood

This repo's `/concepts/` page shows the rail and its folded copy with rings on the line and no square marker behind them.

