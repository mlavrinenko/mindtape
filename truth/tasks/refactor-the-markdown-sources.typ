#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refactor the markdown sources onto one home per fact",
  kind: "docs",
  status: done(
    2026,
    9,
    19,
  )[README renders www/utils/demo.typ; AGENTS rephrased; SKILL.md 320 -> 140 source lines; the audit sits at its three accepted pairs],
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs", "www", "tooling"),
  estimate: (files: 7),
  assignee: "agent",
  x-goals: (
    product: (
      readable-tasks: helps[the three files every agent loads stop paying context for sentences the pages or --help already carry],
      implementable-tasks: neutral,
      enforced-practice: helps[README renders the same module as the site root, so the demo cannot drift again],
      cheap-adoption: helps[a shorter SKILL.md is cheaper context for every adoption attempt],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the audit's last open pairs close or get an accepted reason; docs::md-check keeps the renders honest],
    ),
  ),
)

== Problem

1. The audit's 13 open pairs all sit on README.md, AGENTS.md or a skill
  reference. README keeps its own copy of the opening demo (six pairs), echoes
  AGENTS.md's evaluation sentence and `/concepts/`'s model sentence; AGENTS.md
  repeats `/concepts/`'s "argument shapes it would otherwise drop" phrase.
2. 🔴 `docs/src/skill.typ` sits exactly at its 320-line cap. Its reader is an
  agent that already carries the repo, and most of the file is discoverable
  from `mt --help` or a reference file, so the load buys less than it costs.

== TO-BE

- 🟢 README renders `www/utils/demo.typ` for the opening demo and states the
  model's consequence in its own words.
- 🟢 AGENTS.md keeps every rule and drops the echoed phrasing.
- 🟢 SKILL.md keeps the operating loop, the core rule, project detection and
  the reference pointers; sections `mt --help` or a reference already covers
  go. Every `references/` file stays reachable.
- 🟢 `just dup-prose` leaves only the accepted families (agent-consumed docs,
  shared code), and SKILL.md ends comfortably under its cap.

== Source of truth

- `docs/src/contributing/documentation.typ` — the rules and their named
  exceptions.
- `www/utils/demo.typ` — the demo both front doors render.
- `just dup-prose` — the pair list this task closes.

== Changes

- `docs/src/readme.typ`, `README.md` — the shared demo, the rewritten intro.
- `docs/src/agents.typ`, `AGENTS.md` — the rephrased overlap.
- `docs/src/skill.typ`, `skills/mindtape/SKILL.md` — the shrink.

== Out of scope

- `skills/mindtape/references/*.md` bodies. They are consumed whole by an
  agent and keep their offline-operative minimum.
- The crate render loop shared by `internals.typ` and `agents.typ`; the
  documentation deep dive names it as accepted code.

== Dogfood

`just dup-prose` before and after; `just docs::md` proves each render matches
its source, and `just check` proves every derived fact still resolves.
