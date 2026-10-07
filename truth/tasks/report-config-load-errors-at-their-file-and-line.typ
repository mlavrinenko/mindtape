#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report config load errors at their file and line",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("cli",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[every error says what failed and what to try, in one shape],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 7),
  links: depends-on(
    "build-an-error-dsl-that-fixes-the-message-format-and-the-exit-code.typ",
  ),
)

== Problem

+ Put `[views.broken] sort = ["nofield"]` in `.mindtape/config.toml` and run `mt ls`.
+ 🔴 It prints `error: load /abs/path/.mindtape/config.toml: invalid config: views.broken.sort: unknown sort field: nofield`, an absolute path and no line.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 A config error reads `error: .mindtape/config.toml:12: views.broken.sort: unknown sort field: 'nofield'`, the root-relative file and the line of the key.

== Changes

- `crates/mindtape/src/config/mod.rs`: `ConfigError` carries the file and, from the TOML spans, the line of the key at fault.
- `crates/mindtape/src/config/layers.rs`: the validators' `Result<(), String>` keep their key path and gain the span.
- `crates/mindtape-cli/src/cmd/config.rs` and `crates/mindtape-cli/src/util.rs`'s `open`: no `load <path>` context; the `Diag` names the file.
- `crates/mindtape/src/sort.rs`: `unknown sort field` quotes the field and suggests one.
- `crates/mindtape/src/git/state.rs`: `git could not be run` becomes `can't run git: <reason>`.

== Out of scope

- The validators' own words: the two config-validator tasks.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
