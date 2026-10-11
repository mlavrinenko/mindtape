#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Link each crate's integration tests into one binary",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tests", "size"),
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
      sound-codebase: helps[the e2e suite links every dependency twice instead of 74 times],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ `cargo test --no-run` builds one executable per file in `crates/*/tests/`: 71 for mindtape-cli, 3 for mindtape.
+ 🔴 On 2026-10-11 `target/debug/deps` held 263 executables averaging 298 MB (80 GB), and `just cover` builds the set again, instrumented (51 GB in one worktree). The disk filled.

== TO-BE

- 🟢 Each crate has `tests/it/main.rs` declaring its test files as modules, so `cargo test --no-run` builds one integration binary per crate.
- 🟢 nextest still runs each test in its own process; test names keep their module path under `it`.
- 🟢 Every gate, filter and path list that names `tests/*.rs` follows the move.

== Changes

- `crates/mindtape-cli/tests/*.rs`, `crates/mindtape/tests/*.rs`: moved under `tests/it/`; shared helpers become one `mod` there.
- `crates/*/Cargo.toml`: only if autodiscovery needs a `[[test]]` entry.
- Justfile, `.mmz/config.yaml`, `outdatty.yaml`, `.cargo/mutants.toml`, coverage ignores: paths that name test files.
- Shared helpers declared with `mod common;` in each file become one `mod common;` in `tests/it/main.rs` and `use crate::common::...` in each file. `include_str!`, `include_bytes!` and `#[path]` are relative to the source file, so fix them after the move. `env!("CARGO_MANIFEST_DIR")` paths keep working.
- Nextest filters and overrides that name a binary (`binary(<file>)`) become `binary(it)` plus `test(...)`. Same for Justfile `--test <name>` flags.
- Check: the same test count before and after (`cargo nextest list --workspace`), and one integration executable per crate from `cargo test --no-run`.

== Rollback

Move the files back; nothing outside the test tree depends on the layout.

== Out of scope

- Debug info size, filed host-wide in ~/.nix.
- Inline unit tests, which already share one binary per crate.

== Dogfood

`just check` here runs the suite as two `it` binaries, and `target/` stays a fraction of today's size.

