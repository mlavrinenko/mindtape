#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Keep the docs rules table readable and focus inside the drawer",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("www", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[a reader can scan each recipe and its refusal on a phone without decoding broken words.],
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
      docs-match-code: helps[the focus rings and drawer behavior match the accessibility rules the design documents.],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 23),
  links: edge(
    "surfaced-by",
    "rebuild-the-docs-site-as-a-transit-line.typ",
  )[The critique of the rebuilt site found the mobile table and focus defects.],
)

== AS-IS

+ Open `http://127.0.0.1:8811/mindtape/` at 390 by 844.
+ 🔴 The home rules table keeps its three fixed columns. "Split oversized tasks" prints as "Split oversize d tasks", and a refusal breaks into one-word slivers.

```
table cell widths, 390px viewport
77px  127px  131px
```

+ Tab from the search field. 🔴 Focus lands on pagefind's hidden Clear button, which is fully transparent but still in the tab order.
+ Scroll down and activate Skip to content. 🔴 Main lands at the top of the viewport, under the 99px sticky masthead.
+ Open the drawer at 390 by 844 and Tab past its last link. 🔴 Focus leaves the drawer and reaches the page behind it.


== Environment

- Docs site built from `docs-redesign` at the commit before the fix, in headless Chromium at 390 by 844 and 1440 by 900, light and dark, on Linux. The site has no browser or DOM test harness. Direct browser checks covered the built site.


== TO-BE

- 🟢 At 40rem and below, each recipe row stacks: the recipe link, the effect, and the full refusal on a code panel. No word or code token splits.
- 🟢 The hidden Clear button is out of the keyboard order, and Skip to content leaves 20px below the measured masthead.
- 🟢 An open drawer marks the page inert, keeps Tab between its links and the Menu button, and restores focus to Menu on Escape or a scrim click.

```
cell width at 390px        335px
main top after Skip        119px
masthead bottom             99px
```


== Changes

- `www/assets/styles/components.css`: hide pagefind's suppressed Clear from focus. Stack the home recipe table at 40rem and below, with the refusal on a full-width code panel.
- `www/assets/styles/main.css`: use one green focus ring for buttons, stations and rail links. Measure the masthead for anchor offsets and add the drawer scrim.
- `www/templates/layout.typ`: mark the page inert, add dialog semantics, and trap and restore focus. Add the scrim and give the stacked table its table roles.
- `www/templates/base.typ`: state the home table's stacking exception.


== Out of scope

- The dark `LINE 01` contrast, the 690px reading measure, and the deep-page sidebar scroll. They are separate P2 findings.
- The 4px horizontal overflow at 320px. It is outside the table and predates this fix.


== Dogfood

This is the product's own docs site. Every reader on a phone gets the stacked table, and every keyboard user keeps focus inside the drawer.

