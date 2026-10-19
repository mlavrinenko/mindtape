#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show the daily listing and link checks on the README's first screen",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a reader sees the listing they'd read every day before they scroll],
      typst-over-markdown: helps[the Typst list says why a link can't rot],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 19),
)

== Problem

+ Open `README.md` on GitHub.
+ 🔴 The first screen shows one task file, but not the `mt ls` listing a user reads every day.
+ 🔴 The tagline never says the tool is a tracker, and the Typst list gives no reason links stay valid.


== TO-BE

- 🟢 One sentence and a real `mt ls` run from `examples/api-service` follow the showcase task, at most 100 columns wide and with no warning.
- 🟢 The Typst list says links don't rot, because `mt check` fails a dead one and `mt mv` rewrites every link to a renamed task.
- 🟢 The tagline opens with "Tasks as code. A tracker in your repo, one Typst file per task, checked against your own rules."


== Source of truth

- `www/generated/hero-ls.json`: the `mt ls` run `www/generate-reference.sh` captures in `examples/api-service`.
- `mt check` and `mt mv` run on a copy of `examples/api-service`.


== Changes

- `www/utils/demo.typ`: the listing and its intro under the showcase, and the links reason.
- `www/utils/ui.typ`: read the listing intro from `demo.typ`.
- `www/utils/site.typ`: the new tagline.
- `www/content/index.typ`: the one-sentence summary in the new words.
- `README.md`: rendered again with `just docs md`.


== Out of scope

- The rest of README and the site root, so README stays short.


== Dogfood

Not applicable: the listing shows the fixture's backlog, not this repo's.

