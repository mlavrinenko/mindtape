#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Hint the nearest path-set name in query errors",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
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
        cheap-adoption: helps[a one-letter path-set typo is fixed from the error itself],
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
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 5),
)

== AS-IS

+ Reviewing both waves of the ast-grep milestone after it landed
+ 🔴 #raw("crates/mindtape/src/query/mod.rs"): A mistyped `@name` path set loses the nearest-name hint already computed by `PathFilter::resolve`.

With a `[dicts.source]` set declared and a query glob `@sorce`:

```
$ mt eval numglob
estimate.files: ast-grep rust `warm($$$ARGS)` in `@sorce`: no path set is named `@sorce`
```

`crates/mindtape/src/query/mod.rs` matches `PathSetError::Unknown { name, .. }` and drops `unknown`, which carries the nearest name.

== TO-BE

- 🟢 A mistyped path set names the nearest declared set.
- 🟢 The hint fits the existing error line, without a second paragraph.

```
$ mt eval numglob
estimate.files: ast-grep rust `warm($$$ARGS)` in `@sorce`: no path set is named `@sorce`; did you mean `@source`?
```

== Changes

- `crates/mindtape/src/query/mod.rs`: use `unknown.nearest` when formatting `PathSetError::Unknown`.
- `crates/mindtape/src/query/tests.rs`: a typo test pins the hint and a no-close-name test keeps the plain message.
- `www/content/estimates.typ`: no prose change unless the wording moves.

== Out of scope

- Other path-set errors.
- A new suggestion engine.

== Dogfood

A typo in an estimate or a collector glob gets the same hint as a typo in a tag.
