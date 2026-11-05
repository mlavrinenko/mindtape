#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the bad field in a collector query error",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.95,
    ease: 6,
  ),
  difficulty: 3,
  links: (
    (
      task: "add-ast-grep-queries-as-a-shared-query-engine.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "stateless"),
  x-goals: (
    product: verdicts(
      (implementable-tasks: helps[the error names the exact key to fix]),
      ..(
        "readable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the parser keeps the specific struct's own field errors instead of erasing them],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
)

== AS-IS

+ Reviewing both waves of the ast-grep milestone after it landed
+ 🔴 #raw("crates/mindtape/src/config/collect.rs"): A typo or missing field in a collector query collapses to `data did not match any variant of untagged enum CollectorQuery`.

A collector with `globb = "src/**"` instead of `glob`:

```
$ mt rules
error: .mindtape/config.toml:25:9: can't parse config: data did not match any variant of untagged enum CollectorQuery
```

A missing `pattern` and an extra key produce the same line. `crates/mindtape/src/config/collect.rs` puts the three query shapes in one untagged enum, so serde cannot say which field failed.

== TO-BE

- 🟢 The error names the bad key, or the missing one, as other config keys do.
- 🟢 Regex, Typst and ast-grep queries all keep their current shapes.
- 🟢 A red test covers an unknown key, a missing key and a wrong type.

```
$ mt rules
error: .mindtape/config.toml:25:9: can't parse config: unknown field `globb`, expected one of `language`, `pattern`, `glob`
```

== Changes

- `crates/mindtape/src/config/collect.rs`: implement `Deserialize` for `CollectorQuery` by peeking at `section`, `typst` or `ast_grep` and delegating to the specific struct.
- `crates/mindtape/src/config/collect_tests.rs`: red tests for an unknown key, a missing required key and a wrong type, for each variant.
- `schema/config.schema.json`: re-bless only if the generated shape changes.

== Out of scope

- Changing the accepted config syntax.
- A new query kind.

== Dogfood

A typo in this repo's collector config names the field instead of the enum.
