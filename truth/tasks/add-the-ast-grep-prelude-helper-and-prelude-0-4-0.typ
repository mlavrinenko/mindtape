#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add the ast-grep prelude helper and prelude 0.4.0",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("mt", "tooling", "stateless", "no-repro"),
  x-goals: (
    product: verdicts(
      (
        implementable-tasks: helps[a task writes a shape query with the same helper shape as grep and glob],
      ),
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
        docs-match-code: helps[the supported prelude set names 0.4.0 everywhere it is restated],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 5),
  links: parent("add-ast-grep-queries-as-a-shared-query-engine.typ")
    + depends-on("extract-the-shared-query-engine-and-pin-ast-grep.typ"),
)

== AS-IS

+ Open `lib/prelude.typ` and read the query helpers after `glob(...)`.
+ 🔴 `grep(...)` matches text and `glob(...)` matches paths; no helper builds a shape query.
+ Open `lib/typst.toml`; the package version is 0.3.0, so a task cannot opt into a new helper while an older pin stays loadable.


== TO-BE

- 🟢 `ast_grep(language, pattern, glob?)` builds `(query: "ast-grep", language, pattern, glob)` and accepts a raw block for either string.
- 🟢 `_fmt-query` renders it as `ast-grep rust parse_task($$$ARGS) in crates/**` in `mt eval`.
- 🟢 `lib/typst.toml` and `PRELUDE_VERSION` are 0.4.0, and the 0.3.0 snapshot stays byte-identical.
- 🟢 A current fixture loads 0.4.0, and `PRELUDE_SUPPORTED` names 0.2.0, 0.3.0 and 0.4.0.

```typ
#let ast_grep(language, pattern, ..scope) = (
  query: "ast-grep",
  language: _text(language),
  pattern: _text(pattern),
  glob: scope.pos().at(0, default: none),
)
```


== Changes

- `lib/prelude.typ`: the helper and its `_fmt-query` branch.
- `lib/typst.toml`: version 0.4.0.
- `crates/mindtape/src/loader/preludes/0.4.0/prelude.typ`: the new snapshot, byte-equal to `lib/prelude.typ`.
- `crates/mindtape/src/loader/embedded.rs`: the 0.4.0 constants and snapshot list, keeping 0.3.0 archived.
- `crates/mindtape/src/loader/embedded_tests.rs`: red tests for the 0.4.0 set and the frozen 0.3.0 snapshot.
- `crates/mindtape/src/emit/mod.rs`: emit the current import version.
- `crates/mindtape/src/test_support/fs.rs`: write the current package version.
- `crates/mindtape/src/test_support/mod.rs`: the `typst_import!` macro names the current version.
- `crates/mindtape/src/loader/tests_*.rs`: fixtures that mean "current" follow the bump, while old-pin tests stay.
- `crates/mindtape-cli/tests/it/support/fixtures.rs`: install the current package version.
- `crates/mindtape-cli/tests/it/support/mod.rs`: the unsupported-prelude message names 0.4.0.


== Alternatives

- A keyword argument `ast_grep(language: ..., pattern: ...)`: no other helper takes a keyword for its first value, and `in:` is a Typst keyword, so the positional form is the only one that reads like the others.
- Reusing `grep` with a flag: two engines in one constructor, and the text and shape paths share no compile step.


== Out of scope

- The `Query::AstGrep` parse, compile and match, which land in the next leaf.
- The JSON and import form.
- The `[[tasks.collectors]]` consumer.


== Dogfood

A task opts into 0.4.0 to write `count(0, ast_grep(...))`; the backlog keeps its 0.3.0 pins, which still load.

