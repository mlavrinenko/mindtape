#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Cut AGENTS.md to what mt explain cannot say",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 0),
  tags: ("docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a session reads one shape, not two that disagree.],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the docs stop repeating what the CLI prints.],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the example stops drifting from the config.],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 17),
  links: parent("make-mt-explain-the-session-brief.typ"),
)

== AS-IS

+ Read `AGENTS.md` in this repo.
+ 🔴 Its task example repeats the fields `mt explain` should own, and its `estimate: (days: 1, hours: 2)` is illegal here, where this repo allows only `src-files`.
+ Run `mt explain`.
+ 🔴 The index and `AGENTS.md` each hold half the shape, and neither says which is authoritative.
+ Read `skills/mindtape/SKILL.md`.
+ 🔴 It says run `mt explain` first, but not what to run next, so the session recipe is implicit.

== TO-BE

- 🟢 `AGENTS.md` keeps the repo map, the workspace, the architecture and the mechanism-not-policy rule, and points at `mt explain` for the shape.
- 🟢 The task example shrinks to the `#show: task.with(...)` call, or goes, because `mt add` writes the file and `mt explain` names the fields.
- 🟢 The skill's first section is the recipe: `mt explain`, `mt ls`, `mt find <words>`, `mt add --describe`, `mt add`, `mt check`, and `mt explain rules --for <moment>` before a write.
- 🟢 The recipe lists the moment as `add`, `commit` or `flip`, the values `--for` takes.
- 🟢 `docs/src/agents.typ` and `docs/src/skill.typ` are the sources. `AGENTS.md` and `SKILL.md` regenerate from them.

```text
$ mt explain
project:
  ...
$ mt ls
...
$ mt explain rules --for flip
```

== Changes

- `docs/src/agents.typ`: the shape moves to `mt explain`, and the task example shrinks.
- `docs/src/skill.typ`: the session recipe.
- `AGENTS.md`: regenerated.
- `skills/mindtape/SKILL.md`: regenerated.
- `www/content/agents.typ`: the page states the recipe.

== Alternatives

- Keep the example and the index: two sources for one shape, and the example already drifted on `estimate`.
- Move the example to the skill: the skill installs outside the repo. It cannot state this repo's roots or statuses.
- Generate `AGENTS.md` from `mt explain`: the repo map and architecture are not config, so the generator would need a second source.

== Out of scope

- The repo map, workspace and architecture. They are not config.
- The mechanism-not-policy rule. It is a design principle, not a field.
- The skill's references. They keep the filter language and hand-editing details.
- The `mt add --list` retirement. The sibling task owns it.

== Dogfood

This repo's `AGENTS.md` points at `mt explain` for the shape, and the skill's first section is the recipe.
