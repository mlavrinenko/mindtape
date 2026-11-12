#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Clear the add-hook review nits",
  kind: "dev",
  priority: ice(
    impact: 2,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 1,
  links: (
    (
      task: "let-rules-and-templates-choose-where-new-tasks-land.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "cli", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a small leftover cannot mislead the next reader],
      ),
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
      (
        sound-codebase: helps[the new code carries no copied doc or dead public surface],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 11),
)

== AS-IS

+ Reviewing the final pass over the add-hook code, where small copy-paste remains.
+ 🔴 #raw("crates/mindtape-cli/src/cmd/add/tokens.rs"): `token_name` has `apply_token`'s doc comment, `Derivation::fail`, `type_error` and `field_error` omit the `on.add ` prefix, and `Filter::reads_path_in` has no production caller.

```
/// Apply one `key OP value` token to the fill map.
pub fn token_name(token: &str) -> Result<&str, Diag> {
```

A runtime derive failure prints `fixes: add.typ:fn: ...`, while `error()` prints `on.add fixes: ...`; `type_error` and `field_error` use `context("in", label)` and also omit the prefix. `reads_path_in` is exercised only by `paths_tests.rs`; `check/templates.rs` filters through `Violation.reads`. The doc and stale `allow` predate this commit.


== TO-BE

- 🟢 `token_name` documents what it returns.
- 🟢 Every add-hook error names the hook with the same `on.add {label}` prefix.
- 🟢 An unreleased public method with no caller is gone.

```
error: on.add fixes: add.typ:classify: boom
```


== Changes

- `crates/mindtape-cli/src/cmd/add/tokens.rs`: give `token_name` its own doc and drop the stale `allow`.
- `crates/mindtape-cli/src/cmd/add/hooks.rs`: use the shared `on.add {label}` prefix in `fail`, `type_error` and `field_error`.
- `crates/mindtape/src/filter/mod.rs`: remove `reads_path_in`; it is new in this unreleased commit and has no production caller.
- `crates/mindtape-cli/src/cmd/add/hooks_tests.rs`: pin the derive error prefix.


== Out of scope

- The built-in template name, filed as its own task.


== Dogfood

The next add-hook failure reads like every other hook failure.

