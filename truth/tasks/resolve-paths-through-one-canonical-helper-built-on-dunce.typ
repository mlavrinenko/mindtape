#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Resolve paths through one canonical helper built on dunce",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.85,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 20),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a Windows user sees C:\ paths, not \\?\ ones, and pack errors relative to the project],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[one helper resolves every path, and clippy refuses a bare canonicalize],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 27),
)

== Problem

+ On Windows, run `mt -w <project> ls` (CI run 40580151452, `ls_limit_e2e::a_worktree_flag_pages_with_the_flag_as_typed`).
+ 🔴 The file column prints `\\?\C:\Users\runneradmin\...\tasks\a.typ`, because `Path::canonicalize` returns a verbatim path on Windows.
+ 🔴 A bad key in a pack prints the pack's absolute path, not `team.toml`. The pack path is canonical and the project root isn't, so `strip_prefix` fails. On Windows the temp dir's short `RUNNER~1` name sets this off, and on Linux a project reached through a symlink does.
+ 18 product sites call `canonicalize()` on their own. Two private helpers already strip the verbatim prefix, and two relativize with a bare `strip_prefix`. About 28 Windows tests fail on this split.


== Environment

- `fix-windows-ci` at `19dc0364`, on the `windows-latest` GitHub runner, and on Linux through a symlinked project root.


== TO-BE

- 🟢 Every product path resolves through `mindtape::canonical`, built on `dunce`, so no `\\?\` reaches the user.
- 🟢 A path under the project shows relative to it, whichever spelling of the root the caller holds.
- 🟢 clippy refuses a bare `canonicalize` in new code.


== Changes

- `crates/mindtape/src/config/layers_tests_source.rs`: the red test, a pack under a symlinked root is named `team.toml`.
- `crates/mindtape/src/fs_canonical.rs`: `canonical` and `relative_to`, with tests.
- The 18 `canonicalize()` sites and the `strip_prefix` relativizers in `config`, `rules`, `loader`, `validate`, `model`, `allow` and the CLI: use the helpers.
- `clippy.toml`: disallow `Path::canonicalize` and `std::fs::canonicalize`.
- The tests that compare against a canonical path: build it with `canonical`.


== Out of scope

- How a path prints on Windows, `\` or `/`. The owner decides that.
- The shipped presets' `sh` gates.


== Dogfood

Not applicable. This repo is developed on Linux, without symlinked checkouts.

