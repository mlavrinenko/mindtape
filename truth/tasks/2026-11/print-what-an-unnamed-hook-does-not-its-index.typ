#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Print what an unnamed hook does, not its index",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("stateless", "cli", "docs"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the index says what a gate does, not where it sits in the config],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 19),
)

== AS-IS

+ Run `mt explain` in `examples/api-service`.
+ 🔴 The `flip:` block prints `hooks (error): [0]`, a positional index where every other line carries a name.

```text
$ cd examples/api-service && mt explain
...
flip:
  hooks (error): [0]
rm:
  filter: terminal
```

== Environment

- `mt` at `8503913`, Linux x86_64.
- `examples/api-service`, whose `[[on.flip]]` gate omits `name`.

== TO-BE

- 🟢 An unnamed hook prints the command it runs.
- 🟢 A gate with no command prints the message it carries.
- 🟢 A hook with neither prints its `on.<moment>[<index>]` label.
- 🟢 An unnamed add hook prints the directory it files into.

```text
$ cd examples/api-service && mt explain
...
flip:
  hooks (error): scripts/test
rm:
  filter: terminal
```

== Changes

- `crates/mindtape/src/config/active_rules.rs`: carry the command a hook action runs.
- `crates/mindtape-cli/src/cmd/explain/index/mod.rs`: print it for an unnamed hook, and the directory for an unnamed add hook.
- `crates/mindtape-cli/src/cmd/schema/registry.rs`: `rules` moves to version 2.
- `www/content/cli/explain.typ`: the line an unnamed hook prints.
- `CHANGELOG.md`: the entry under `[Unreleased]`.

== Out of scope

- `mt explain rules`. Its `(no message set)` column stays: a gate with no message prints nothing on a breach.
- The `refusals:` block. It already prints the command.

== Dogfood

This repo gives every hook a name, and its index reads the same. `examples/api-service` shows the command.
