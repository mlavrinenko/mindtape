#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Gate the ast-grep collector examples",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  links: (
    (
      task: "add-ast-grep-queries-as-a-shared-query-engine.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "www", "docs", "tests", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[the documented shape is parsed by a gate instead of trusted],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the page's example is the fixture the gate reads],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 5),
)

== AS-IS

+ Reviewing both waves of the ast-grep milestone after it landed
+ 🔴 #raw("www/content/rules.typ"): Both ast-grep collector examples carry `docs:hand-typed` and no fixture parses them.

`www/content/rules.typ` and `www/content/config.typ` both carry `// docs:hand-typed the ... shape is a sketch until ... carries one`, and `examples/` contains no ast-grep collector. A typo in either block would not fail any gate.

== TO-BE

- 🟢 At least one ast-grep collector block comes from a checked example project.
- 🟢 The rules page renders the collector and its reader from the same config slice.
- 🟢 The hand-typed marker is gone, and the docs block gate still passes.

```
$ just docs blocks
ok
```

== Changes

- `examples/rules/.mindtape/config.toml`: add a checked ast-grep collector and a rule that reads it, under a `docs:slice rules/ast-grep` marker.
- `www/content/rules.typ`: render the slice and add the key to `checked-slices`.
- `www/content/config.typ`: either slice the collector from the same fixture or keep one marked sketch with a reason.
- `www/generate-examples.sh`: no change unless the example is added to a new fact.

== Out of scope

- A new example project.
- Changing the collector behavior.

== Dogfood

`just check-examples` and the docs slice gate prove the primary collector example parses.
