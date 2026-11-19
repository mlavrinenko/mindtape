#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Let a filter expression decide whether add verifies",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 4,
  tags: ("stateless", "mt", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[an agent's add refuses an incomplete task.],
      config-over-code: helps[the policy is a filter expression, not a Rust mode enum.],
      cheap-adoption: helps[one config line sets the policy for every project.],
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the config page and the templates page read the same key.],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 20),
)

== AS-IS

+ Run `mt add "Sort tasks without a due date last"` in a terminal, with no `--set`.
+ 🔴 It refuses the add and lists every unfilled slot:

```
$ mt add "Sort tasks without a due date last"
error: missing required slots: 'kind', 'assignee', 'as-is', 'to-be', 'changes', 'out-of-scope', 'dogfood'
hint: pass each with --set name=value, or with -f
hint: run mt add --describe to see the slots
```

+ Read `mt explain` and `.mindtape/config.toml`. `[add]` holds `editor`, `interactive`, `header` and `comments`. Each one changes the editor, and the refusal stays.

== TO-BE

- 🟢 `[add] verify-when` takes a filter expression over the invocation's ambient facts, and `!tty()` verifies only a run without a terminal.
- 🟢 An absent `verify-when` verifies always. A project that says nothing keeps today's refusal.
- 🟢 `--verify` and `--no-verify` override the expression for one call.
- 🟢 A run that skips verification leaves an unfilled required slot as its `slot(...)` call, skips `pre_verify`, and skips every `[[on.add]]` hook.
- 🟢 The task writes, and a note names `mt check`.

```
$ mt add "Sort tasks without a due date last"
wrote truth/tasks/2026-11/sort-tasks-without-a-due-date-last.typ (a3d)
note: filed without verification; `mt check` lists what is missing
```

== Changes

- `crates/mindtape/src/config/add.rs`: `verify_when: Option<String>`, parsed with `Condition::parse` at load.
- `crates/mindtape/src/template/fill/mod.rs`: a mode that leaves an unfilled required slot in place instead of `FillError::Missing`.
- `crates/mindtape-cli/src/cmd/add/pipeline.rs`: evaluate `verify-when` against a `ConditionEnv`, and skip `pre_verify` and the hooks when it says no.
- `crates/mindtape-cli/src/cmd/add/mod.rs`: `--verify` and `--no-verify`.
- `www/content/templates.typ`, `www/content/config.typ`, `CHANGELOG.md`: the key and what a skipped verification leaves behind.

== Alternatives

- A `verify = "always" | "never" | "auto"` enum: a fixed set of modes where a filter expression covers every case.
- Tying verification to `[add] interactive`: the editor and the verification are separate decisions, and each has its own key.

== Out of scope

- Running `[[on.add]]` hooks over an incomplete task. A hook reads fields the fill has not written, and a skipped verification runs none.
- A status or tag for an incomplete task. A project writes its own `when` conditions, such as `!has_tag("draft")`.

== Dogfood

This repo sets `[add] verify-when = "!tty()"` in the global defaults. A terminal add files an incomplete task, and an agent's add still refuses one.
