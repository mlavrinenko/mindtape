#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Finish the path-set hint at every query site",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.98,
    ease: 7,
  ),
  difficulty: 2,
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
      (docs-match-code: helps[every path-set site names the same suggestion]),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
)

== AS-IS

+ Verifying the fixes for the ast-grep review
+ 🔴 #raw("crates/mindtape/src/estimate_queries.rs"): A config-level `[tasks.estimate.queries]` path set still drops the nearest-name hint, and a collector hint omits the `@`.

With `[dicts.source]` declared and the unit path set set to `@sorce`:

```
$ mt check
error: .mindtape/config.toml:12: tasks.estimate.queries.files: no path set is named `@sorce`
```

A collector with the same typo prints `hint: did you mean source?` without the `@`. `estimate_queries::validate_config` returns plain strings, and `PathFilter::resolve` builds `Unknown` from bare dictionary names.

== TO-BE

- 🟢 A config-level path-set typo gets the same nearest-name hint as a query glob.
- 🟢 A path-set hint spells the name with its `@`, as the query text does.
- 🟢 A red test covers the config-level unit and the collector.

```
$ mt check
error: .mindtape/config.toml:12: tasks.estimate.queries.files: no path set is named `@sorce`
hint: did you mean @source?
```

== Changes

- `crates/mindtape/src/estimate_queries.rs`: return `Problem`s from `validate_config` and keep their `unknown`.
- `crates/mindtape/src/config/semantic.rs`: stop flattening those problems with `plain`.
- `crates/mindtape/src/git/state.rs`: build the path-set `Unknown` with `@`-prefixed names.
- `crates/mindtape/src/estimate_queries_tests.rs`: a red test for the config-level hint and the `@` spelling.

== Out of scope

- Redesigning `suggest::Unknown`.
- Changing other config errors' hint rendering.

== Dogfood

A typo in this repo's estimate path set gets the same hint as a typo in a query glob.
