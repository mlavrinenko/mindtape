#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Expose ast-grep through the collector consumer",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("mt", "gating", "stateless", "no-repro"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a rule reads a code fact as `derived.<name>`, like any other collector field],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the collector reuses the query engine and doesn't add a second file walk],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
  links: parent("add-ast-grep-queries-as-a-shared-query-engine.typ")
    + depends-on("carry-the-ast-grep-form-through-json-and-import.typ"),
)

== AS-IS

+ Read `[[tasks.collectors]]` in `crates/mindtape/src/config/collect.rs`.
+ 🔴 A collector query is a body-section regex or a Typst function. It cannot read a code fact.
+ So a rule cannot read `derived.<name>` for a call-site count.


== TO-BE

- 🟢 A collector query `{ ast_grep = { language, pattern, glob? } }` reuses the query engine and returns `["path"]`.
- 🟢 The collector has no path set of its own. A rule reads `count_paths(derived.<name>, "@source")`, as it does for any path list.
- 🟢 `mt rules` shows the collector kind and its readers.
- 🟢 A bad pattern or unknown language is a collector problem, as a bad regex is today.

```toml
[[tasks.collectors]]
name = "callers"
query = { ast_grep = { language = "rust", pattern = "parse_task($$$ARGS)" } }
returns = ["path"]
```


== Changes

- `crates/mindtape/src/config/collect.rs`: the `ast_grep` query variant and load-time validation.
- `crates/mindtape/src/collect/astgrep.rs`: run the shared engine and map its result.
- `crates/mindtape/src/collect/mod.rs`: dispatch the new variant.
- `crates/mindtape/src/config/collect_tests.rs`: red parse and validation tests.
- `crates/mindtape/src/collect/tests.rs`: result and problem tests.
- `www/content/config.typ`: the collector shape.
- `www/content/rules.typ`: the derived-field example.
- `CHANGELOG.md`: the collector entry.


== Alternatives

- A separate `[[tasks.derive]]` table: the collector contract already states its field, so a second table is one concept too many.
- A Typst function that shells out to `sg`: needs `mt allow` and varies by host.


== Out of scope

- Extra language grammars and a Typst grammar.
- A persistent parse cache stays out. The process cache is enough until a profile says otherwise.


== Dogfood

A budget rule reads real call sites as `derived.callers` instead of a text proxy.

