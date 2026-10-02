#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Lint the rendered Markdown for typlite traps and fix the current hits",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("docs", "tooling"),
  estimate: (src-files: 3),
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
      docs-match-code: helps[a render defect fails the gate instead of reaching GitHub],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 2),
  links: parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Open `skills/mindtape/SKILL.md` on GitHub and find the "Filter and sort" row.
+ 🔴 The `|` in `` `--sort <field>[:asc|desc]` `` splits the row into an extra cell. `CONTRIBUTING.md:63` breaks the same way.
+ 🔴 `CONTRIBUTING.md:77` reads "in docs/contributing/gates.md the gated example projects": the source glued a `;` to a `#link(...)[...]`, and the render dropped it. `CONTRIBUTING.md:85` has the same defect.
+ 🔴 `docs/contributing/gates.md:37` repeats its last sentence. `commit-messages.md:13`, `gates.md:26` and `mutation-testing.md:25` break a code span across a line.
+ The Gate column shows `docs-check` and `docs-md-check`; the recipes are `docs::check` and `docs::md-check`.
+ `docs/src/lib.typ` lists these traps, but nothing checks for them.


== TO-BE

- 🟢 `just docs md-check` also lints the rendered `.md` files and fails, naming file and line, on: a `|` inside a code span in a table row, an odd number of backticks on a line, an escaped backtick, and `-\>`.
- 🟢 It also scans `docs/src/**` for a `#link(...)[...]` with a `;` glued to it.
- 🟢 The trap list lives once, in `docs/contributing/generated-docs.md`, and `docs/src/lib.typ` links to it.


== Changes

- `.just/scripts/docs-md-check.sh`: add the lint and the source scan. Run it on today's tree first; it must list the hits above.
- `www/utils/gate-notes.typ`: escape the `|`.
- `www/gates.jq`: print the real `docs::` recipe names.
- `docs/src/skill.typ`, `docs/src/contributing.typ`, `docs/src/contributing/{gates,commit-messages,mutation-testing}.typ`: fix the hits.
- `docs/src/lib.typ`, `docs/src/contributing/generated-docs.typ`: merge the two trap lists into `generated-docs.typ`; `lib.typ` keeps a link.


== Rollback

Revert the commit. The lint is one block in `docs-md-check.sh`; the source fixes stand on their own.


== Out of scope

- Rewriting these files' prose: the CONTRIBUTING and SKILL tasks do that.


== Dogfood

`just check` runs `docs::md-check`, so every commit here passes the lint.

