#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Let a condition ask whether stdin is a terminal",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt", "cli", "docs", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 26),
)

== AS-IS

A harness with a terminal on stdout and `/dev/null` on stdin passes for a person.

+ Run `mt add` from a harness whose stdout is a PTY and whose stdin is `/dev/null`, such as Zed's native agent terminal.
+ 🔴 `mt` files the task without verification and without its `[[on.add]]` hook, because `!tty()` is false.

```
$ mt add "Half a terminal"
wrote tasks/half-a-terminal.typ (qhm)
next: mt eval qhm
note: filed without verification; `mt check` lists what is missing
```

The same run with `--verify` lands in `tasks/month/`, so the hook and the check are both reachable. The condition is what cannot say so.


== TO-BE

- 🟢 The condition language exposes the terminal state of stdin as `stdin_tty()`. A project writes `verify-when = "!tty() || !stdin_tty()"`.
- 🟢 `tty()` keeps meaning stdout, so column visibility, colour and hyperlinks answer the question they answer today.
- 🟢 The function table says which stream each predicate reads.

```
$ mt add "Half a terminal"
on.add file-by-month: dir = tasks/month
wrote tasks/month/half-a-terminal.typ (abc)
```


== Changes

- `crates/mindtape/src/filter/condition_env.rs`: a `stdin_tty` field, and its `probe()` default.
- `crates/mindtape/src/filter/condition.rs`: the `stdin_tty` predicate beside `tty`, and the module doc's predicate list.
- `crates/mindtape-cli/src/cmd/ls/rules.rs`: `build_env` fills the field from `std::io::stdin().is_terminal()`.
- `crates/mindtape/src/filter/condition_tests.rs`: the predicate's two answers, and its refusal of an argument.
- `www/content/cli/ls.typ`: the function table gains `stdin_tty()`, and `tty()` says stdout.
- `www/content/templates.typ`: the `verify-when` section names the half-terminal case.
- `.mindtape/config.toml`: this repo's own `verify-when` becomes `!tty() || !stdin_tty()`.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Out of scope

- `stderr_tty()`. `diag/render.rs` reads stderr for colour, but no config key does, so the fact has no consumer yet.
- Renaming `tty()` to `stdout_tty()`. It appears in the docs and in this repo's config. The rename waits for a breaking window.
- A compound `interactive()` predicate. The project composes the policy from the facts, as `[ls.columns.file] when` already does.


== Dogfood

This repo's own `verify-when` becomes `!tty() || !stdin_tty()`, so filing from Zed's agent terminal verifies and runs `file-by-month`.


== Goals

+ #Helps(adopter, to: run-checks-at-the-write)[a harness that cannot answer a prompt gets the refusal, and one that can still files leniently].
+ #Helps(maintainer, to: match-docs-to-code)[the `verify-when` page stops claiming every agent gets the refusal].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe).

