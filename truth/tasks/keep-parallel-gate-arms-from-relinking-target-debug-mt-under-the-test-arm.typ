#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep parallel gate arms from relinking target/debug/mt under the test arm",
  priority: ice(
    impact: 4,
    confidence: 0.6,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling", "tests"),
  estimate: (src-files: 2),
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
      sound-codebase: helps[a green gate stops depending on arm timing],
      docs-match-code: neutral,
    ),
  ),
  kind: "dev",
  assignee: "agent",
  status: proposed(2026, 10, 1),
)

== Problem

+ Run `just check` on a tree where every arm is stale.
+ 🔴 On 2026-10-01 the `test` arm failed `add_e2e::end_to_end_files_from_yaml_on_stdin` with `Failed to spawn Command { ... "/home/user/mindtape/target/debug/mt" ... }: No such file or directory (os error 2)`; the re-run passed with no change.
+ Suspects: `check-examples` (`Justfile:200`), `check-doc-coverage` and the docs arm each build `mt` while `test` runs off `target/debug/mt`; cargo's lock covers a build, not nextest's run phase. Unconfirmed: `strace` shows a fresh build makes no link call, so the window needs two dirty builds with different fingerprints in one run. Only a stale tree gives that, hence never seen on warm runs.


== TO-BE

- 🟢 No gate arm relinks `target/debug/mt` while `test` runs; a cold `just check` cannot fail on a vanished binary.


== Changes

- First reproduce: watch `target/debug/mt`'s inode (`inotifywait -m target/debug`) across a `just check` after a `crates/` edit, and name the arm whose build relinks it.
- `Justfile` `check`: build the workspace once, sequentially, before the `[parallel]` fan-out, so every arm's later build is fresh and never relinks. No extra disk, and the build is paid anyway. A separate target dir per arm is the fallback only: another ~2.6 GB (what `target/mt-dev` holds) and a second compile on every source change.

== Rollback

Drop the sequential pre-build from `check`.


== Out of scope

- Serialising the arms: parallelism is the gate's speed.


== Dogfood

Every `just check` in this repo, and every delegated worktree's gate before `just integrate`.

