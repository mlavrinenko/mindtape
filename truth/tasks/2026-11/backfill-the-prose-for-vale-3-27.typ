#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Backfill the prose for Vale 3.27",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("docs", "gating", "tooling", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[the backlog reads under the same reader as the docs],
      implementable-tasks: neutral,
      enforced-practice: helps[one Vale version holds every prose gate],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the docs page states the one version the shell builds],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 14),
  reviewed: on(2026, 11, 14),
)

== AS-IS

+ Run `vale --no-global --config=.config/vale.ini --output=line docs/src www/content www/utils | wc -l`. It prints `48`.
+ Run `vale --no-global --config=.config/vale.ini --output=line truth/tasks | wc -l`. It prints `99`.
+ 🔴 Vale 3.27 reports those 147 alerts, and the prose gates still call `vale-prose`, a second 3.26 build in `nix/dev/vale-prose.nix`. `.just/docs.just` and `.githooks/pre-commit` hold the call.

```
$ vale --no-global --config=.config/vale.ini --output=line docs/src www/content www/utils | wc -l
48
$ vale --no-global --config=.config/vale.ini --output=line truth/tasks | wc -l
99
```

== TO-BE

- 🟢 Both commands print `0`, and every prose gate calls `vale` 3.27.
- 🟢 The change deletes `nix/dev/vale-prose.nix` and drops `valeProse` from the shell. One binary runs the docs, the task bodies and the code comments.
- 🟢 The full gate passes over the docs, the site sources and the backlog with a single Vale version.
- 🟢 `docs/src/contributing/documentation.typ` states one Vale version, and the generated pages match.
- 🟢 The `docs::vale-tasks` comment in `.mmz/config.yaml` describes the rule set `.config/vale.ini` holds.

```
$ vale --no-global --config=.config/vale.ini --output=line docs/src www/content www/utils truth/tasks | wc -l
0
```

== Changes

- `.just/docs.just`: call `vale` in `docs::vale-prose` and `docs::vale-tasks`, the red test first.
- `docs/src/*.typ`, `www/content/*.typ` and `www/utils/*.typ`: clear the 48 alerts, led by `Prose.ConsequenceClause` (15) and `Prose.PassiveVoice` (9).
- `truth/tasks/*/*.typ`: clear the 99 alerts, led by `Prose.PassiveVoice` (27), `Prose.ServesAsDodge` (21) and `Prose.ConsequenceClause` (19).
- `.githooks/pre-commit`: call `vale` for the task prose.
- `nix/dev/shell.nix`: drop the `valeProse` build and the comment above `vale`, and delete `nix/dev/vale-prose.nix`.
- `.config/outdatty.yaml`: remove `nix/dev/vale-prose.nix` from the two Vale groups.
- `.mmz/config.yaml`: drop the `[truth/tasks/*/*.typ]` clause from the `docs::vale-tasks` comment.
- `docs/src/contributing/documentation.typ`: state one Vale version and drop the pin paragraph.
- `docs/src/contributing/gates.typ`: give the pre-commit list the `vale` call.

== Alternatives

- Keep the `vale-prose` 3.26 pin. Two binaries then run over the same prose, and the docs page explains why one gate differs from the rest.
- Set the 13 rules those alerts name to `warning`. That hides 147 alerts and weakens the rule set the repo chose.
- Rewrap each flagged paragraph for the 3.26 sentence split. The next rewrap turns the alerts back on.

== Out of scope

- Code comments: `lint-code-comments-with-vale` owns the first prose rules there.
- Commit messages: `lint-commit-messages-with-vale` owns that gate.
- New Vale rules beyond the 13 the 147 alerts name.

== Dogfood

This repo's `just check` runs `docs::vale-prose` and `docs::vale-tasks` through `vale` 3.27. A green run over the same tree shows the one binary covers every prose gate.
