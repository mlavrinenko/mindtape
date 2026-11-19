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
  status: proposed(2026, 11, 19),
)

== AS-IS

+ Reviewing the bare `mt explain` index while removing its `absent: none` line
+ 🔴 #raw("crates/mindtape-cli/src/cmd/explain/index.rs"): `absent:` lists the keys a project leaves open without saying what each absence permits, while `tags: any` states the consequence inline

```
$ cd examples/minimal && mt explain
...
commits:
  trailer: Refs
absent: actors, estimate, kinds, comments, extensions
rules: none
derived fields: none
tags: any
```

`tags: any` says what an empty `[tasks.tags] allowed` permits. `absent: kinds` says only that the key is unset, so a reader has to open `www/content/config.typ` to learn that any kind is legal.

== TO-BE

- 🟢 Each absent key carries what its absence permits, as `tags: any` already does.
- 🟢 An unset `kinds` accepts any kind, and the index says so.

```text
$ cd examples/minimal && mt explain
...
commits:
  trailer: Refs
absent: actors (none declared), estimate (any unit), kinds (any kind),
  comments (allowed), extensions (none)
```

== Changes

- `crates/mindtape/src/explain.rs`: pair each absent key with the consequence its default carries.
- `crates/mindtape-cli/src/cmd/explain/index.rs`: print the pair.
- `www/content/cli/explain.typ`: the parenthetical each absent key carries.

== Out of scope

- The JSON. `absent` stays a list of key names. A consumer keeps reading the same array.
- The `refusals:` block. It already states its consequence.

== Dogfood

This repo constrains every key, so it doesn't print an `absent:` line. `examples/minimal` shows the pairs.
