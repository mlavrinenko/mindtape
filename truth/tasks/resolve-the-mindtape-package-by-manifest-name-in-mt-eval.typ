#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "resolve the mindtape package by manifest name in mt eval",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 4),
  assignee: "agent",
  status: wip(
    2026,
    9,
    11,
  )[Confirmed on main: eval.rs:96 checks only that lib/typst.toml exists.],
)

== Summary

`crates/mindtape-cli/src/eval.rs:96` registers `<root>/lib` as the
`@local/mindtape` package whenever a `lib/typst.toml` merely *exists*, never
reading the name that file declares. In a foreign worktree whose `lib/` is
some other Typst package, `mt eval` therefore resolves `@local/mindtape` to
that directory and fails with `package manifest contains mismatched name
'<other>'`, while `mt ls` and `mt check` on the same tree succeed.

The batch loader already has the guard: `crates/mindtape/src/loader.rs:322`
`mindtape_lib_dir` returns the directory only when the manifest says
`name = "mindtape"`, and its doc comment states the reason in as many words.
`eval.rs` is a second, hand-written copy of the same decision that never
learned it — so the two drifted, and copying the check across would only set
up the next drift.

Two more gaps on the same seam: `eval::build_world` never loads the project
config, so `[[typst.local-packages]]` overrides do not apply, and it never
calls `with_package_root`, so an unoverridden `@local/...` resolves against
the platform data directory rather than `$MINDTAPE_DATA_HOME` the way every
loader-routed command does.


== Scope

- Export `loader::package_overrides` and `loader::package_root` from
  `crates/mindtape`, and have `eval::build_world` call them instead of
  deciding for itself, so there is one answer to "which `@local` packages
  does this project resolve".
- Resolve the project root with `find_root` and load its `Config`, so the
  `[[typst.local-packages]]` entries `mt ls` honours reach `mt eval` too.
- Regression tests in `crates/mindtape-cli/src/eval_tests.rs`: a foreign
  `lib/typst.toml` declaring another name must not be hijacked, and an
  in-tree `name = "mindtape"` lib must still be registered.

