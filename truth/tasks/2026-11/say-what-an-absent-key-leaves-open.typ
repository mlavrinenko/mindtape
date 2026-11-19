#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Say what an absent key leaves open",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 2,
  links: (
    (
      task: "omit-the-absent-line-when-nothing-is-absent.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "cli", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the index says what an unset key permits, not only that it is unset],
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

+ Run `mt explain` in `examples/minimal`.
+ 🔴 The index lists each key the project leaves open without saying what its absence permits, while `tags: any` states the consequence inline.

```text
$ cd examples/minimal && mt explain
...
commits:
  trailer: Refs
absent: actors, estimate, kinds, comments, extensions
rules: none
derived fields: none
tags: any
```

`tags: any` says what an empty `[tasks.tags] allowed` permits. `absent: kinds` says only that the key is unset, so a reader has to open `www/content/config.typ` to learn that any kind is legal. The index carries two vocabularies for "unset", and only one of them states the consequence.

== TO-BE

- 🟢 Each open key prints in its own block, with what its absence permits.
- 🟢 `any` for a free vocabulary, `none` for an empty registry, and the policy word for comments.
- 🟢 `any` reads as the key's own domain, and `kinds: any` already says kind.
- 🟢 `absent:` leaves the text. The JSON keeps the array.

```text
$ cd examples/minimal && mt explain
project:
  ...
  actors: none
task:
  ...
  estimate: any
  kinds: any
  comments: allowed
  extensions: none
  ...
tags: any
```

A second `absent:` block carrying `kinds (any kind)` would repeat the key in the noun, and `any kind` says kind twice.

== Changes

- `crates/mindtape-cli/src/cmd/explain/index/render.rs`: print each open key in its block with its consequence, and drop the `absent:` line.
- `crates/mindtape-cli/src/cmd/explain/index_tests.rs`: a project that leaves every key open prints `statuses: any`, `actors: none`, `estimate: any`, `kinds: any`, `comments: allowed` and `extensions: none`.
- `www/content/cli/explain.typ`: the inline consequence each open key carries.
- `CHANGELOG.md`: the entry under `[Unreleased]`.

== Out of scope

- The JSON. `absent` stays a list of key names, empty when nothing is open.
- The `refusals:` block. It already states its consequence.

== Dogfood

This repo constrains every key, and nothing prints `any`. `examples/minimal` shows the six lines.
