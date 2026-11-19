#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Mark a command the user has not allowed in place",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  kind: "dev",
  tags: ("stateless", "cli", "docs"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[a reader meets each command once, with the one word that says what it does],
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
+ 🔴 One command prints twice, once in the block that runs it and again in a `refusals:` block that repeats it with `(not allowed)`.

```text
$ cd examples/api-service && mt explain
...
flip:
  hooks (error): scripts/test
refusals:
  waits: scripts/locked {id} (not allowed; ask the user to run `mt allow`)
  on.flip: scripts/test (not allowed; ask the user to run `mt allow`)
```

== TO-BE

- 🟢 A command the user has not allowed prints once, in the block that runs it.
- 🟢 Its label carries `not allowed` in place of the severity. A line carries one word.
- 🟢 The `[[tasks.waits]]` checkers print under `task:` as `waits:`.
- 🟢 One `allow:` line under `more:` names the command that records the approval.

```text
$ cd examples/api-service && mt explain
...
  waits (not allowed): scripts/locked {id}
...
flip:
  hooks (not allowed): scripts/test
rm:
  filter: terminal
...
more: mt explain <add|rules|tags|views|dicts|templates>
allow: mt allow records the approval for each command marked not allowed
```

== Changes

- `crates/mindtape-cli/src/cmd/explain/index/mod.rs`: bucket each rule by label, and carry the waits.
- `crates/mindtape-cli/src/cmd/explain/index/render.rs`: print the label, the waits and the `allow:` line.
- `crates/mindtape-cli/src/render/objects.rs`: `waits` replaces `refusals`.
- `www/content/cli/explain.typ`: the label and the `allow:` line.
- `CHANGELOG.md`: the entry under `[Unreleased]`.

== Out of scope

- `mt explain rules`. Its `(no message set)` column stays.
- The approval digest. `mt allow` still approves every command at once.

== Dogfood

This repo allows every command, and its index prints neither the label nor the `allow:` line. `examples/api-service` shows both.
