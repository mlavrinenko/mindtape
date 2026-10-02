#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Cut CONTRIBUTING to the rules and each gate note to the problem it stops",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (src-files: 12),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent session loads a third of today's CONTRIBUTING and finds each rule once],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[each gate says in one clause what breaks without it],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
  links: depends-on(
    "write-plain-prose-rules-for-the-docs-and-check-for-history-and-shouting.typ",
    "lint-the-rendered-markdown-for-typlite-traps-and-fix-the-current-hits.typ",
  )
    + parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Run `wc -c CONTRIBUTING.md`.
+ 🔴 It prints `29837`. The gate table is 18.5 KB of that, and every agent session here loads the whole file through `AGENTS.md`.
+ 🔴 The Notes cells for `fmt-check`, `test`, `check-secrets` and `script-scope-check` are 200-300 word arguments. `docs/contributing/gates.md` or the script header already makes most of them.
+ Several rules appear in two or three files: no `Refs:` on a filing commit (`CONTRIBUTING.md:111`, `commit-messages.md:16`, `SKILL.md:67`); fork before flipping (`CONTRIBUTING.md:81`, `commit-messages.md:20`).
+ The deep dives narrate history: `gates.md:43-49` calls itself "not a current reading".


== TO-BE

- 🟢 Each gate's note is one clause naming the problem the gate stops. The argument moves to the script header.
- 🟢 Each process rule is stated once; other files link to it.
- 🟢 `.linecop.yaml` caps the rendered `CONTRIBUTING.md`, so its size cannot creep back.
- 🟢 CONTRIBUTING and the deep dives follow the plain-prose rules.


== Source of truth

- `Justfile`, `.just/*.just` and `.just/scripts/*.sh`: what each gate does.
- `docs/contributing/documentation.md`: the plain-prose rules.


== Changes

- `www/utils/gate-notes.typ`: one clause per gate.
- `.just/scripts/*.sh`: headers take any argument cut from a note that the header lacks.
- `docs/src/contributing.typ`: rewrite whole. Also fix "one of those worktrees" (no antecedent), `HarvestError` (it lives in the typst-harvest crate; use `RecurError`), and the release sentence: `just release` refuses a non-empty `[Unreleased]`.
- `docs/src/contributing/*.typ`: rewrite whole, drop duplicated rules and the history passages.
- `.linecop.yaml`: a cap on `CONTRIBUTING.md`.


== Out of scope

- What the gates do: this task changes their docs only.


== Dogfood

Every session in this repo loads the shorter CONTRIBUTING.md.

