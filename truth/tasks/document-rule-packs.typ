#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Document rule packs",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt", "docs"),
  estimate: (files: 3),
  kind: "docs",
  assignee: "agent",
  status: wip(2026, 9, 16),
  links: depends-on("ship-the-task-practice-as-the-mindtape-tasks-pack.typ"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a reader finds the one-line adoption on the config page],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
)

== Problem

+ Open `www/content/config.typ` at "The four layers".
+ 🔴 It explains layers whose arrays replace wholesale and says nothing about `extends`, `disable`, rule names or the `mindtape-tasks` pack, so a reader cannot find the one-line adoption.


== TO-BE

- 🟢 `www/content/config.typ` shows `extends` and `disable` with one embedded and one path pack, how pack rules append, and how a same-named project rule replaces one.
- 🟢 It lists the rules in `mindtape-tasks` by name.

== Source of truth

- `crates/mindtape/src/config_packs.rs` and `schema/config.schema.json`: every key, default and error the page states.
- `mt config show --origin` run in this repo: the origin lines the page quotes.


== Changes

- `www/content/config.typ`: a packs section after "The four layers".
- `CHANGELOG.md`: an entry under Unreleased.
- `docs/src/skill.typ`: one line saying `mt config show --origin` names the pack a rule came from; `just docs md` regenerates `skills/mindtape/SKILL.md`.


== Out of scope

- Any change to how packs behave.


== Dogfood

Not applicable: docs only. The pack is dogfooded in `ship-the-task-practice-as-the-mindtape-tasks-pack.typ`.

