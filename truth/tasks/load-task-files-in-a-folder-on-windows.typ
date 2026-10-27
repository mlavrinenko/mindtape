#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Load task files in a folder on Windows",
  kind: "fix",
  priority: ice(
    impact: 10,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a Windows user's tasks in a folder load at all],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[one constructor builds every task file's Typst world],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 27),
)

== Problem

+ On Windows, run `mt ls` in a project whose root is `tasks/`.
+ 🔴 Every task fails: `error: tasks\alpha.typ: bad virtual path "tasks\\alpha.typ": path contains backslash`.
+ `HarvestWorld::with_root`, from typst-world 0.5.5, names the main file by the platform's relative path, and Typst 0.15 refuses `\` in a virtual path. The loader, `mt eval` and the prelude tests build their world with it.
+ CI run 40573274304 failed 302 tests that print this error, and more that load nothing because of it.


== Environment

- `main` at `364f88ad` plus `d8bd87c4`, on the `windows-latest` GitHub runner.


== TO-BE

- 🟢 A task file in any folder loads on Windows, and its main id is `/tasks/alpha.typ` as on Linux.


== Changes

- `crates/mindtape/src/loader/world_tests.rs`: the red test, a file in a folder gets a `/`-separated main id. It's red only on Windows, as are the 302 tests above.
- `crates/mindtape/src/loader/world.rs`: `disk_world`, which builds the main id from the path's components joined with `/`, and `typst_path`, moved here.
- `crates/mindtape/src/loader/mod.rs`, `crates/mindtape/src/validate/candidate.rs`, `crates/mindtape-cli/src/cmd/eval.rs`, `crates/mindtape/src/loader/tests_prelude.rs`: build every world with `disk_world`.


== Out of scope

- The fix in typst-world itself, where `World::disk` should call `VirtualPath::virtualize`. It needs a typst-world release, which is the owner's call.
- The other Windows failures in the same run. Each cause gets its own task once this one stops hiding them.


== Dogfood

Not applicable. This repo is developed on Linux, where the paths already use `/`.

