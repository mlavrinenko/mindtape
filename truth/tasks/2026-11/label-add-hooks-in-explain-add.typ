#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Label add hooks in mt explain add",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  links: (
    (
      task: "merge-and-explain-on-add-hooks-across-layers-and-packs.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "cli", "tests", "stateless"),
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
+ 🔴 #raw("crates/mindtape-cli/src/cmd/explain/add.rs"): `ActiveAddHook.label` was computed and never used, so an unnamed hook printed `-` and JSON had no label.

```
on.add[0]  template == "task"  dir = "tasks/fixes"  built-in
```

The text still printed `-`, and `--json` had only `name`, which is `null` for an unnamed hook.


== TO-BE

- 🟢 Text and JSON show `on.add.<name>` or `on.add[<index>]`.
- 🟢 The JSON schema records `label` beside `name`.

```json
{"label": "on.add.fixes", "name": "fixes", "when": "template == \"task\""}
```


== Changes

- `crates/mindtape/src/config/active_add.rs`: label a named hook by name when no origin is recorded.
- `crates/mindtape/src/config/active_add_tests.rs`, `crates/mindtape-cli/src/cmd/explain/add_tests.rs`: pin the label.
- `crates/mindtape-cli/src/cmd/explain/add.rs`, `tests/it/explain_add_e2e.rs`: print and expose it.
- `schema/output/explain-add/1.schema.json`, `www/content/cli/explain.typ`: document the field.


== Out of scope

- `mt config show --origin`, which already labels hooks.


== Dogfood

`mt explain add` names a hook exactly as `mt config show --origin` does.

