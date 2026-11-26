#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Keep code in section-head titles looking like inline code",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("www", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[a title's `mt ls` reads as the command it names],
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
  assignee: "agent",
  status: done(2026, 11, 23),
  links: edge(
    "surfaced-by",
    "rebuild-the-docs-site-as-a-transit-line.typ",
  )[The rebuilt site stripped the code chrome in section heads.],
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/` in Chromium.
+ 🔴 The section head "And here's how `mt ls` lists it" renders `mt ls` as bare mono text. It has no code paper, border or padding, while inline code in the body has all three.
+ 🔴 The stylesheet removes that chrome for this one title role:

```css
.section-head h2 code { font-size: 0.72em; background: none; border: 0; padding: 0; }
```


== Environment

- The docs site built from `main` at the commit that filed this, in Chromium 155 on Linux.
- Both the light and the dark theme, through the site's theme switch.


== TO-BE

- 🟢 A code span in a section-head title carries the inline-code chrome body prose uses: the mono face, code paper, 1px code border and padding.
- 🟢 The span takes its colors from `--code-bg`, `--code-fg` and `--code-border`, so it stays visible in both themes.
- 🟢 The home section head and an article h2 set their code spans at the same `0.78em`, with normal letter spacing.
- 🟢 Every other element on the page keeps its current position.

```css
.section-head h2 code { font-size: 0.78em; letter-spacing: 0; }
/* light: background #f3f5f0, border #ccd6ce */
/* dark:  background #1a261e, border #2c3a32 */
```


== Changes

- `www/assets/styles/components.css`: replace the `.section-head h2 code` reset with the heading-scale rule, so the shared inline-code chrome shows. No test covers site CSS, so check the built page in both themes.


== Out of scope

- Restyling inline code elsewhere, including the borderless tint the design critique floats. This task restores the code look in titles only.
- A CSS regression gate. The site has none, so check this change by eye in both themes.
- `DESIGN.md`. This change leaves the type tokens alone.


== Dogfood

This repository's home page section head renders `mt ls` as inline code in both themes, and `just docs::check` builds the page.

