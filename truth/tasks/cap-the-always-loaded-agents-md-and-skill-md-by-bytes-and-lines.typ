#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Cap the always-loaded AGENTS.md and SKILL.md by bytes and lines",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent session pays a bounded, known cost for AGENTS.md and the skill],
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
  status: done(2026, 10, 3),
)

== Problem

+ Read the overrides in `.linecop.yaml`.
+ 🔴 `./AGENTS.md` and `./skills/mindtape/SKILL.md` carry `exclude: true`, so their rendered size goes unchecked, though every agent session loads them whole. Only their Typst sources carry a line cap, which a paragraph growing into an essay passes.


== TO-BE

- 🟢 `just check-file-size` fails when AGENTS.md or SKILL.md outgrows a line or byte budget, as it does for CONTRIBUTING.md.


== Changes

- `.linecop.yaml`: replace `exclude: true` on `./AGENTS.md` and `./skills/mindtape/SKILL.md` with `limit` and `max_bytes` at their size plus about 10%, and say so in the comment.


== Rollback

Revert the commit; both files return to `exclude: true`.

== Out of scope

- README.md, which no agent session loads whole.


== Dogfood

Every agent session here loads both files, so their budgets bound what each session spends on them.
