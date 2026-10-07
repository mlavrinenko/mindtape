#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the agents and skill doc sources under their caps",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("size", "docs"),
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
      sound-codebase: helps[the files every session loads have room for the next rule without a rewording pass],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 7)[both sources and renders under 90%, caps unchanged],
  links: depends-on(
    "fold-rendered-docs-into-their-source-in-the-size-sweep.typ",
  ),
)

== Problem

+ Run `linecop --baseline 90`.
+ 🔴 `docs/src/agents.typ` stands at 158 of 166 lines and its `AGENTS.md` at 75 of 78; `docs/src/skill.typ` at 131 of 144 and its `SKILL.md` at 69 of 76.


== TO-BE

- 🟢 `linecop --baseline 90` names none of the four, and no cap was raised.


== Source of truth

- `docs/contributing/documentation.md` "Rewrite the whole file": how the rewrite is done.
- `.linecop.yaml`: the caps, which stay as they are.


== Changes

- `docs/src/agents.typ`: rewrite whole per "Rewrite the whole file" in `docs/contributing/documentation.md`, moving reasons into a deep dive.
- `docs/src/skill.typ`: the same, moving detail into `skills/mindtape/references/`.
- The deep dive or reference that receives a moved section.
- `AGENTS.md`, `skills/mindtape/SKILL.md`: re-rendered by `just docs md`.


== Out of scope

- `CONTRIBUTING.md` and `README.md`: not flagged.


== Dogfood

Every agent session here loads the shorter files.
