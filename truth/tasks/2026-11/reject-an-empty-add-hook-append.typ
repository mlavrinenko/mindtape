#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Reject an empty add-hook append",
  kind: "dev",
  priority: ice(
    impact: 2,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  links: (
    (
      task: "let-rules-and-templates-choose-where-new-tasks-land.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "cli", "stateless"),
  x-goals: (
    product: verdicts(
      (:),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  status: done(2026, 11, 12),
  assignee: "agent",
)

== AS-IS

+ Reviewing the on-add branch after `just check`.
+ 🔴 #raw("crates/mindtape/src/config/add_hooks.rs"): `append = { tags = [] }` loads, fires, and reports `append tags` while changing nothing.

```toml
[[on.add]]
actions = [{ append = { tags = [] } }]
```

`mt add` prints `on.add[0]: append tags` and the field keeps its old value.


== TO-BE

- 🟢 An append with no items is a config error, like an action with no kind.

```
error: .mindtape/config.toml:5: on.add[0].actions[0].append.tags: append needs at least one item
```


== Changes

- `crates/mindtape/src/config/add_hooks.rs`: refuse an empty append and drop the ignored render loop.
- `crates/mindtape/src/config/add_hooks_tests.rs`: pin the refusal.


== Out of scope

- Empty `set` lists, which legitimately clear a field.


== Dogfood

A no-op hook action fails at config load.

