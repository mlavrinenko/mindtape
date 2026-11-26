#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Drop the page count from the article rail",
  kind: "fix",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
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
      sound-codebase: helps[a docs build that brings the page count back fails in `just docs::check`],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 25),
)

== AS-IS

+ Run `just docs::build`.
+ Open an article page's built HTML, such as `www/public/mindtape/cli/index.html`, and read the rail's meta block.
+ 🔴 The block lists the section name, a page count and the version.

```html
<div class="rail-meta"><span>REFERENCE</span><span>4 PAGES</span><span>Version 0.6.0</span></div>
```


== Environment

- `mt` built from `main` at `ce789032`, with tola 0.7.1 on Linux.


== TO-BE

- 🟢 The rail's meta block shows the section name and the version.
- 🟢 The rail shows only those two.

```html
<div class="rail-meta"><span>REFERENCE</span><span>Version 0.6.0</span></div>
```


== Changes

- `www/post-build.sh`: the red test first, an arm that fails the build when a rail's meta block holds a page count.
- `www/templates/layout.typ`: the rail renders the group title and `Version`. The page-count span goes, with the comment that names it.


== Out of scope

- The rail's CSS in `www/assets/styles/components.css`. `.rail-meta` keeps its column, gap and top border for the two remaining lines.
- The sidebar and the route strip don't count pages.


== Dogfood

`just docs::build` and `just docs::check` run the new arm, and every built article page loses the page-count line.

