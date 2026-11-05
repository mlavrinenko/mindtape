#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show the ast-grep query in mt rules",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 1,
  links: (
    (
      task: "add-ast-grep-queries-as-a-shared-query-engine.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "cli", "stateless"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[the listing names the query instead of repeating one opaque kind],
      ),
      ..(
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the command shows the same shape the config page documents],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
)

== AS-IS

+ Reviewing both waves of the ast-grep milestone after it landed
+ 🔴 #raw("crates/mindtape/src/config/active_rules.rs"): `mt rules` prints two ast-grep collectors identically because the derived-field detail is `None`.

Two collectors with different patterns render the same line:

```
derived fields
  [path]  derived.callers-a  ast-grep from .../config.toml  read by no rule
  [path]  derived.callers-b  ast-grep from .../config.toml  read by no rule
```

`crates/mindtape/src/config/active_rules.rs` sets `detail: None` for `AstGrep`, while a Typst collector shows `module::function`.

== TO-BE

- 🟢 A line names the language, pattern and optional glob the collector reads.
- 🟢 A red test pins two different collectors to different lines.

```
derived fields
  [path]  derived.callers-a  ast-grep rust `parse_task($$$ARGS)` from .../config.toml  read by rules.a
  [path]  derived.callers-b  ast-grep rust `warm($$$ARGS)` in `src/**` from .../config.toml  read by rules.b
```

== Changes

- `crates/mindtape/src/config/active_rules.rs`: set `detail` for `AstGrep` to the language, pattern and optional glob.
- `crates/mindtape/src/config/active_rules_tests.rs` and `crates/mindtape-cli/src/cmd/rules_tests.rs`: pin the new detail.
- `www/content/config.typ`: say `mt rules` names the shape query.

== Out of scope

- Changing the Typst detail.
- A new JSON key; `detail` already exists.

== Dogfood

`mt rules` tells a reader which call site a collector reads without opening the config.
