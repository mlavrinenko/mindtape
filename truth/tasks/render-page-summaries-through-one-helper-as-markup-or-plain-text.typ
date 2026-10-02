#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Render page summaries through one helper as markup or plain text",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("www", "docs"),
  estimate: (src-files: 4),
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
      docs-match-code: helps[a summary renders the same way on every surface that shows it],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
  links: parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Read the Commit rules line of `README.md`'s doc index.
+ 🔴 It shows a backslash before each backtick around `Refs` and `mt check --commit-msg`. The site's lede and the page's meta description show the backticks as text.
+ The cause: a summary is a plain string. `www/templates/layout.typ:269`, `:138` and `docs/src/lib.typ`'s `www-summary` print it as text, and `commit-rules.typ:8` is the only summary with code in it.
+ The other summaries name code without formatting (`mt ls`, `every()`, `tasks.estimate`). They run from 8 words to three sentences (`rule-packs.typ:10`).


== TO-BE

- 🟢 A summary is Typst markup. One helper turns it into content for the site and into plain text for `<meta>` and README.
- 🟢 Code names in summaries are in backticks.
- 🟢 `www/generate-site-pages.sh` fails on a summary longer than one sentence or 120 characters, naming the page.


== Changes

- `www/utils/site.typ`: the helper, `summary(route, plain: bool)`.
- `www/templates/layout.typ`: lede and `<meta description>` through the helper.
- `docs/src/lib.typ`: `www-summary` through the helper.
- `www/generate-site-pages.sh`: the length check. Run it on today's pages first; it must name `rule-packs`.
- `www/content/*.typ`: rewrite the summaries.


== Rollback

Revert the commit; summaries go back to plain strings.


== Out of scope

- Config schema descriptions: a separate task fixes their wording at the source.


== Dogfood

This repo's README and site render through the helper on every `just check`.

