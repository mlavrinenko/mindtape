#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Bump the canonical schema for the ast-grep query tag",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
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
      (
        cheap-adoption: helps[a mixed-version export/import gets the designed upgrade hint],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the version constant follows its own documented rule],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
)

== AS-IS

+ Reviewing both waves of the ast-grep milestone after it landed
+ 🔴 #raw("crates/mindtape/src/model/canonical.rs"): `SCHEMA_VERSION` stays 1 while `mt ls --json` can now emit an `ast-grep` query an older `mt import` cannot parse.

`crates/mindtape/src/model/canonical.rs` declares `pub const SCHEMA_VERSION: u32 = 1;` and the new `CanonicalQuery::AstGrep` variant sits beside the old tags. The module comment says to bump when a shape change could break `mt import`, and `crates/mindtape-cli/src/cmd/import/input.rs` implements the friendly newer-version check. A 0.4 export still says `"schema_version": 1`, so an older importer fails in the enum instead of the version check.

== TO-BE

- 🟢 A current export says `"schema_version": 2`.
- 🟢 An older binary refuses with the existing upgrade hint, not an unknown-variant serde error.
- 🟢 The version tests use the constant, so the bump is one line.

```
$ mt eval a --json | jq .schema_version
2
```

== Changes

- `crates/mindtape/src/model/canonical.rs`: bump `SCHEMA_VERSION` to 2 and update the comment.
- `crates/mindtape/src/model/canonical_tests.rs`: pin the new version where a literal remains.
- `www/content/schema.typ`: no text change unless it names the number.

== Out of scope

- Per-command schema versions.
- Changing the record shape again.

== Dogfood

Exporting from this repo and importing on an older binary points at the upgrade instead of an unknown variant.
