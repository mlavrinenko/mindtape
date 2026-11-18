#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Show the project's shape in mt explain",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 1),
  tags: ("cli", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[the index reads as the project's shape, not a bare list.],
      implementable-tasks: helps[a session starts without reading `mt config show` or a stale example.],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the CLI answers what the project is, not only what it checks.],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[AGENTS.md stops repeating what `mt explain` prints.],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 19),
  links: parent("make-mt-explain-the-session-brief.typ"),
)

== AS-IS

+ Run `mt explain` in this repo.
+ 🔴 It lists the rules, tags, views and dicts. It omits the project's roots, statuses, actors and `rm` policy. It also omits the task's priority, difficulty, estimate, kinds, links, comments, extensions and templates, and the commit trailer.
+ Run `mt config show`.
+ 🔴 The answer is 900 lines of TOML, and the shape sits across nine sections.
+ Copy the `estimate: (days: 1, hours: 2)` example from `AGENTS.md`.
+ 🔴 `mt check` refuses it, because this repo allows only `src-files`.

== TO-BE

- 🟢 Bare `mt explain` starts with `project:`, `task:` and `commits:` blocks, plus `absent:` for unconfigured keys.
- 🟢 Every line is config-only. The index stays at 93ms.
- 🟢 The statuses group by `open` and `terminal`. Extensions print the actual `x-` names. Templates mark the default.
- 🟢 The JSON envelope carries the same blocks.

```text
project:
  roots: truth/tasks (tag: mt)
  statuses:
    open: proposed, accepted, wip
    terminal: done, cancelled, failed
  actors: max (human, maintainer), agent (llm)
  rm: terminal only
task:
  priority: ice (impact, confidence, ease)
  difficulty: 0-9
  estimate: src-files (optional)
  kinds: feat, fix, refactor, docs, test, dev, decision, milestone
  links: depends-on, parent, recurred-from, supersedes, duplicates, related, surfaced-by
  comments: forbidden
  extensions: x-goals (give-every-goal-verdict)
  templates: decision, dev, ..., tweak (default: task)
commits:
  trailer: Refs
absent: none
```

== Changes

- `crates/mindtape-cli/src/cmd/explain/index.rs`: the `project`, `task`, `commits` and `absent` blocks, their JSON, and the tests.
- `schema/output/explain/2.schema.json` (new): the blocks, and keep `1.schema.json`.
- `www/content/cli/explain.typ`: the blocks and their fields.
- `crates/mindtape-cli/tests/it/explain_e2e.rs`: the blocks in text and JSON.

== Alternatives

- A `config:` block that dumps `mt config show`: 900 lines, and the reader still has to find the shape.
- A separate `mt explain shape` listing: one more command for the first question of a session.
- A single `shape:` block: the word already means the body's heading structure. The split gives each block its own scope.
- A `statuses: proposed, accepted, wip, done (terminal), ...` line: the repetition the grouping removes.
- `sections` and `kind sections` lines: the per-kind sections live in `when` strings, not config, so deriving them would evaluate filters.
- An `x-fields:` line: the project says `x-<ns>`, and the pattern is cryptic. `extensions:` prints the actual names.

== Out of scope

- Counts and the next task. `mt ls` and `mt board` own the state.
- The slots of each template. `mt add -t <name> --describe` owns them.
- The `commits.run` command. `mt config show` owns it.
- The pack origins. `mt config show --origin` owns them.
- The `refusals:` block and the `templates` listing. Their sibling tasks own them.
- The `sections` and `kind sections` lines. They are not config.

== Dogfood

This repo's `mt explain` prints the shape an agent needs before `mt ls`, and `AGENTS.md` stops repeating it.
