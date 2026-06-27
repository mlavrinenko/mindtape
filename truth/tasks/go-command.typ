#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "`go` command — open a task in the configured editor by handle",
  status: done(2026, 6, 28),
  priority: ice(impact: 6, confidence: 0.8, ease: 7),
  tags: ("cli",),
)

== Summary

`mindtape go <task>` opens the task file in the configured editor. `<task>` is
any ref `resolve_task` already accepts — handle (`1b`), slug, or path — so the
workflow is `mindtape ls` then `mindtape go 1b`.

== Config

```toml
[go]
editor = "zeditor {task}"
```

`{task}` is substituted with the resolved file path. When the editor command
carries no `{task}` token, the path is appended as a trailing argument (covers
plain `nvim`). The command is split on whitespace into program + args; `{task}`
is replaced per token.

== Editor resolution

`[go] editor` → `$VISUAL` → `$EDITOR` → error. No silent `vi` fallback: `go` is
launch-oriented and a GUI user dropped into `vi` is a worse failure than a clear
error naming `[go] editor`. (This differs from `[add] editor`, whose terminal
read-back fill justifies the `vi` floor.)

== Scope

- `[go]` config section (`GoConfig { editor: Option<String> }`), library-side
  alongside `config_add.rs`; shape only, format-blind.
- `go` subcommand: resolve the ref, build the editor command, spawn it, wait
  (`.status()`) so terminal editors hold the foreground and GUI editors that fork
  return immediately. Error on a non-success exit.
- Pure command-builder helper (template + path → argv, append-when-absent) with
  unit tests; `{task}` substitution and the append fallback are the testable core.

== Notes

- Fire-and-forget vs read-back: `go` never reads the buffer back, so it does NOT
  reuse `add --edit`'s blocking read path. A `zeditor {task}` that returns
  instantly is correct for `go` but would break `add --edit` — keep them separate.
- Sibling to `flip-status-command.typ`; both lean on `resolve_task`.
