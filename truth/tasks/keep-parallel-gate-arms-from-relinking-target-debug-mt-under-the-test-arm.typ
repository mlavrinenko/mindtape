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
+ `check-examples` (`Justfile:200`, `cargo run -p mindtape-cli`) and `check-doc-coverage` (`.just/scripts/check-doc-coverage.sh:14`, `cargo build -p mindtape-cli`) write `target/debug/mt` while `test` runs e2e tests off it. Cargo's build lock covers the build, not nextest's run phase, and a `-p` build may resolve features differently from the `--workspace` one, so it relinks the binary out from under a running test. `Justfile:397` already records the same race for the rrule arm.


== TO-BE

- 🟢 No gate arm other than `test` writes `target/debug/mt`; a cold `just check` cannot fail on a vanished binary.


== Changes

- `Justfile` `check-examples` and `.just/scripts/check-doc-coverage.sh`: build into their own target dir (as `build-mt` does with `target/mt-dev`) and run that binary; first confirm the relink by watching `target/debug/mt`'s inode across a cold `just check`.

== Rollback

Point both arms back at `target/debug/mt`.


== Out of scope

- Serialising the arms: parallelism is the gate's speed.


== Dogfood

Every `just check` in this repo, and every delegated worktree's gate before `just integrate`.

