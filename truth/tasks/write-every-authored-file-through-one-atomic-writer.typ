#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write every authored file through one atomic writer",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("mt", "cli"),
  estimate: (files: 6),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a lint, not review, keeps new write sites on the atomic path],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[a task file is either the old bytes or the new ones, never a torn half],
    ),
    process: (
      sound-codebase: helps[one sanctioned write primitive instead of a dozen raw `fs::write` calls],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 9, 28),
)

== Problem

+ `rg -n 'fs::write' crates/*/src --glob '!*tests*'` lists the authored-file writes: `write_task` in `crates/mindtape-cli/src/util.rs:309` (nine callers: `flip`, `set`, `link`, `mv`, `recur`, `recur_advance`, `review`, `import`), plus `add.rs:206`, `init.rs`, `install.rs`, `preset_cmd.rs`.
+ 🔴 Each is `std::fs::write`: truncate, then write. A process killed between the two (Ctrl-C, OOM, a full disk) leaves an empty or partial `.typ`, and a concurrent reader (tinymist, a hook's `mt ls`, an editor reload) can read it half-written.
+ Nothing stops the next write site from being another raw `fs::write`; `crates/mindtape/src/cache.rs:252` already hand-rolls temp-then-rename for itself.


== Environment

- `HEAD` (`f505a12`), every platform.


== TO-BE

- 🟢 Every authored-file write goes through one `write_atomic(path, bytes)`: write a sibling temp file in the same directory, fsync, rename over the target. A killed `mt flip` leaves the old file whole.
- 🟢 `mt add` creates with `create_new`, so it can never overwrite a file that appeared after its dedup check.
- 🟢 `std::fs::write` and `std::fs::File::create` are `disallowed-methods` in `clippy.toml`; the one sanctioned call inside `write_atomic` carries the only `#[allow]`, so a new raw write fails `just check` naming the helper to use.


== Changes

- `crates/mindtape/src/` (new `fs_atomic.rs` or beside `cache.rs`): `write_atomic`, and `cache.rs` reuses it; the red test asserts no temp file survives and that a failed write leaves the target unchanged.
- `crates/mindtape-cli/src/util.rs`: `write_task` calls `write_atomic`.
- `crates/mindtape-cli/src/add.rs`: create with `create_new`.
- `crates/mindtape-cli/src/init.rs`, `install.rs`, `preset_cmd.rs`: switch to `write_atomic`.
- `clippy.toml`: `disallowed-methods` for `std::fs::write` and `std::fs::File::create`, with test code exempted at each crate root (`cfg_attr(test, allow(...))`) and per integration-test crate.


== Out of scope

- Lost updates between two writers of one file (a lock or compare-and-swap): with one agent per worktree it has no reported case; file it when one appears.
- `mt rm`'s writes: they go through the external `typst-cleanup` crate, which carries its own apply.


== Dogfood

Every `mt` write in this repo takes the atomic path; the lint guards every future write site.

