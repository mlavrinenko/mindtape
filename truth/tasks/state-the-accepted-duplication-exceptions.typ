#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "State the accepted duplication exceptions",
  kind: "docs",
  status: wip(
    2026,
    9,
    18,
  )[the ledger's deliberate pairs get a rule and /agents/ delegates the envelope],
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("docs", "www"),
  estimate: (files: 4),
  assignee: "agent",
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[the reader learns which findings the audit will not chase, so a future pair does not get re-litigated],
      enforced-practice: helps[documentation.typ carries the exception beside the rule it limits],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the ledger's accepted pairs point at the rule that accepts them instead of looking like missed work],
    ),
  ),
)

== Problem

1. `just dup-prose` closes every pair this wave can: 13 remain, and all but
  three are README/AGENTS work or the front-door demo. Three findings are
  deliberate and unnamed by any rule:
  - `www/content/templates.typ` ~ `skills/mindtape/references/add-slots.md`
    (the positional-is-title rule).
  - `www/utils/ui.typ` ~ `docs/src/agents.typ` (the tagline).
  - `www/content/internals.typ` ~ `docs/src/agents.typ` (a crate render loop).
2. 🔴 `www/content/agents.typ` restates `/schema/`'s envelope shape
  (`schema_version`, `applied`, facet attribution, `superseded`), the pair the
  adjudication sent to the schema page.

== TO-BE

- 🟢 `documentation.typ` names the two deliberate families: docs consumed
  whole keep their offline-operative minimum, and a shared block of code is
  not prose slop.
- 🟢 `/agents/` keeps only the agent-facing consequence of the envelope and
  links `/schema/` for its shape.
- 🟢 `just dup-prose` after the change shows those pairs with their reason in
  the doc, not as open findings.

== Source of truth

- `docs/src/contributing/documentation.typ` — the rule the exceptions limit.
- `www/content/schema.typ` — owns the envelope keys, including `applied`.
- `just dup-prose` — the pair list.

== Changes

- `docs/src/contributing/documentation.typ` — one exception paragraph.
- `docs/contributing/documentation.md` — regenerated render.
- `www/content/agents.typ` — the envelope paragraph becomes a pointer plus
  the bail-out consequence.

== Out of scope

- Sharing the crate render loop through `www/utils/crate-notes.typ`; the
  markdown wave touches both sides and can extract it then.
- The README side of the demo partial, and the remaining README/AGENTS pairs.

== Dogfood

The exception paragraph and the audit run against each other: a finding it
names stops being a finding.
