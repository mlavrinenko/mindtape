#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Omit the absent line when nothing is absent",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("cli", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[the first screen of a session loses a line that says nothing.],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the index prints only what the project actually constrains.],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the page stops explaining a line the reader never sees.],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 20),
  links: parent("make-mt-explain-the-session-brief.typ"),
)

== AS-IS

+ Run `mt explain` in this repo.
+ 🔴 The shape ends on `absent: none`, a line that lists nothing.
+ Run `mt explain` in `examples/minimal`.
+ 🔴 The same line prints `absent: actors, estimate, kinds, comments, extensions`. The block's presence tracks the project rather than the reader's need.

```text
$ mt explain | sed -n '1,20p'
...
commits:
  trailer: Refs
absent: none
rules:
```

== TO-BE

- 🟢 A project that constrains every key doesn't print an `absent:` line.
- 🟢 A project that leaves a key open still prints `absent:` with the keys. `refusals:` works the same way: it prints only when a gate refuses.
- 🟢 `--json` keeps `absent` as an array, empty when nothing is absent. A consumer reads one key either way.

```text
$ mt explain
...
commits:
  trailer: Refs
rules:
  tasks.rules: ...
```

== Changes

- `crates/mindtape-cli/src/cmd/explain/index.rs`: print `absent:` only when the list is non-empty.
- `crates/mindtape-cli/src/cmd/explain/index_tests.rs`: the red test first, a project that constrains every key not printing a line.
- `www/content/cli/explain.typ`: the `absent:` line prints only when a key is open.
- `CHANGELOG.md`: the entry under `[Unreleased]`.

== Alternatives

- Print `absent: none`: the line the reader skips, and the one this task removes.
- Drop `absent` from the JSON too: a consumer would branch on the key's presence, and an empty array already says it.
- Fold each absent key into its block as `statuses: (default)`: the block would then carry a value the config does not set, and the reader cannot tell a default from a declaration.

== Out of scope

- What each absence means. `absent: kinds` still lists the key without saying any kind is legal. That is its own task.
- The `refusals:` block. It already prints only when a gate refuses.

== Dogfood

This repo constrains every key, so its `mt explain` loses the line. `examples/minimal` keeps it.
