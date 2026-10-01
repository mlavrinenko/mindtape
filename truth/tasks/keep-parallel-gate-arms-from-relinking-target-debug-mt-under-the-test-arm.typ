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
      sound-codebase: helps[a green gate stops depending on arm timing],
      docs-match-code: neutral,
    ),
  ),
  kind: "dev",
  assignee: "agent",
  status: done(2026, 10, 2),
)

== Problem

+ Run `just check` on a tree where every arm is stale.
+ 🔴 On 2026-10-01 the `test` arm failed `add_e2e::end_to_end_files_from_yaml_on_stdin` with `Failed to spawn ... target/debug/mt ...: No such file or directory`; the re-run passed.
+ Reproduced 2026-10-02 (inode watch on `target/debug/mt`, three stale runs): `mt` is one path for two builds. `mindtape-cli`'s dev-dependencies enable `test-support` features, so `cargo test --no-run` links a different unit than a plain `cargo build -p mindtape-cli`/`cargo run`. Cargo swaps the uplift hard link whenever it points at another inode, even for a fresh unit (delete, then link: ~85 ms of ENOENT). A plain build (`check-doc-coverage`, the docs arms, `check-examples`) swapped `mt` back at the instant nextest began running tests, and once 18 s before.


== TO-BE

- 🟢 Every `mt` build in the gate resolves the unit the test arm links, so no arm swaps `target/debug/mt` once `test` runs.


== Changes

- The planned sequential pre-build is not enough: later builds of the other feature set still swap the link back, fresh or not.
- `check-examples`, `check-doc-coverage.sh` and `www/generate.sh` build with `cargo build -p mindtape-cli --tests`, the invocation that resolves the test arm's unit; `check-examples` then runs `./target/debug/mt`, as `cargo run` takes no `--tests`.
- Measured after: one relink per stale run, during the test arm's own build, never after (two stale runs).
- Argued in `docs/contributing/gates.md`, "One mt binary, one feature set".
- No red test: a gate arm that probes the link would swap it under the test arm itself.

== Rollback

Restore the plain `cargo build`/`cargo run` in those three places.


== Out of scope

- Serialising the arms: parallelism is the gate's speed.


== Dogfood

Every `just check` in this repo, and every delegated worktree's gate before `just integrate`.

