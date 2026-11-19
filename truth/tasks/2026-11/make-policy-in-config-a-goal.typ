#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Make policy-in-config a goal",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 3,
  tags: ("stateless", "mt", "docs"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a task rates a hardcoded policy against a goal.],
      config-over-code: helps[the goal states that a policy belongs in config.],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[AGENTS.md renders the goals from their source.],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 20),
)

== AS-IS

+ Read `docs/src/agents.typ`. `== Mechanism, not policy` sits under `= Task definition language`, so it reads as a rule about writing checks.
+ Read `truth/goals.typ`. The product goals cover task quality, checks, adoption, the medium, the files and the CLI.
+ 🔴 The goals leave out the rule that a policy belongs in config. A task that hardcodes one has no verdict to give it:

```
$ grep -c "config" truth/goals.typ
0
```

+ The principle lives in one section of one file, and a session that skips that section misses it.

== TO-BE

- 🟢 `truth/goals.typ` carries `config-over-code`. A task rates a hardcoded policy against a goal that states it.
- 🟢 `docs/src/agents.typ` renders both goal files. Every session reads the goals without a section of its own.
- 🟢 The goal's acid-test carries the rule, and `= Mechanism, not policy` disappears.

```
Policy lives in config, not code: Every rule, status, tag, path and threshold `mt` enforces is a config key. A project changes one by editing `.mindtape/config.toml`, and no Rust change is needed. A behavior only a Rust change can alter fails this goal.
```

== Changes

- `truth/goals.typ`: `config-over-code`, after `enforced-practice`.
- `.mindtape/templates/*.typ`: one `config-over-code` slot per template, so `mt add` fills a verdict for it.
- `docs/src/agents.typ`: import both goal files, render them under `= Goals`, and drop `== Mechanism, not policy`.
- `AGENTS.md`: the render.
- `.config/outdatty.lock`: the two template groups.

== Alternatives

- Keep `== Mechanism, not policy` as a section: a session that skips it misses the rule, and no task rates it.
- Add the goal to `truth/process-goals.typ`: the acid-test is about what a project can change, which is the product.

== Out of scope

- The `[add] verify-when` work. `let-a-filter-expression-decide-whether-add-verifies.typ` carries it.
- The AGENTS.md size cap. The Goals section fits by trimming the repo map and the Workspace section.

== Dogfood

This repo's open tasks rate `config-over-code`, and `mt check` refuses one that leaves it out.
