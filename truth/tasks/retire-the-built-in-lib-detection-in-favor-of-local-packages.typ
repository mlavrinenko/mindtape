#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Retire the built-in lib detection in favor of local-packages",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("mt",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the core names no project path, and this repo resolves its prelude through the mechanism every project uses],
      docs-match-code: helps[the config page states one way to pin a package, not a rule plus an exception],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 26),
)

== Problem

+ Read `package_overrides` in `crates/mindtape/src/loader/mod.rs`.
+ 🔴 `mindtape_lib_dir` resolves `@local/mindtape` from `<root>/lib` whenever its `typst.toml` names the package `mindtape`, a path and a package name written into the core library.
+ Read `www/content/config.typ` under Local Typst packages.
+ 🔴 It documents `[[typst.local-packages]]` as the way to pin a package, then names this hardcoded rule as the exception "this repository" uses.


== TO-BE

- 🟢 `package_overrides` returns only the `[[typst.local-packages]]` entries, so the core names no directory or package.
- 🟢 This repo's `.mindtape/config.toml` pins `mindtape` to `lib`, and every checkout and worktree still evaluates its tasks against its own `lib/`.
- 🟢 A project whose `lib/` holds the `mindtape` package and has no entry reads the installed package, like any other project.


== Changes

- `crates/mindtape/src/loader/mod.rs`: drop `mindtape_lib_dir` and its branch in `package_overrides`. The red test first: a project with a `mindtape` `lib/` and no entry resolves the machine's package.
- `crates/mindtape/src/loader/tests*.rs`, `crates/mindtape-cli/src/cmd/eval_tests.rs`: the fixtures that lean on the implicit `lib/` declare the entry.
- `.mindtape/config.toml`: add `[[typst.local-packages]]` with `name = "mindtape"` and `path = "lib"`.
- `crates/mindtape-cli/src/cmd/eval.rs`: its comment on `package_overrides` stops naming the `lib/` rule.
- `www/content/config.typ`: drop the exception sentence. `CHANGELOG.md`: an entry under `[Unreleased]`.


== Out of scope

- The examples under `examples/`, which already declare their entries.
- Installing the prelude: `mt install` writes the machine directory as before.


== Dogfood

This repo's own backlog becomes the first project in it that pins its prelude through `[[typst.local-packages]]`, the way the docs tell every other project to.
