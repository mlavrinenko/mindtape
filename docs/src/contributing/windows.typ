// docs/src/contributing/windows.typ renders docs/contributing/windows.md
// through docs/generate-md.sh (typlite; docs/src/lib.typ lists the
// constraints).
//
// The Rust channel below reads www/generated/crate-map.json, which takes it
// from rust-toolchain.toml, so a toolchain bump updates the sentence.
#import "../lib.typ": doc-title, fact, just

#metadata((
  output: "docs/contributing/windows.md",
)) <mindtape-md>

#doc-title[Windows and the pty tests]

What runs on Windows, what cannot, and how to catch a Windows compile error
before CI does.

= What CI runs on Windows

The `windows` job in `.github/workflows/ci.yml` runs `cargo build` and
`cargo nextest run`. It does not run #just.check(), because the other gates
behave the same on every platform. Windows has no Nix, so
`taiki-e/install-action` installs nextest there. The job covers the code that
differs by platform: `cmd /C` against `sh -c` for an `[[on.flip]]` `run`, the
config and data paths from `dirs`, and path separators.

On Windows, `dirs` reads the data path from a system API that no environment
variable moves, so a test that sets `XDG_DATA_HOME` would test a path `mt`
never reads there. That is why `MINDTAPE_CONFIG_HOME` and `MINDTAPE_DATA_HOME`
exist. `tests/install_e2e.rs` sets them to check both where `mt install`
writes the `@local/mindtape` package and where loading a project finds it.

= The pty tests are Unix-only

`tests/interactive_e2e.rs` drives `mt ls --interactive` under a real terminal
through `script(1)`, which the dev shell provides. It needs a terminal to
exercise the `guard` checks, `event_loop` and `draw`. It and
`tests/tty_e2e.rs` are `#![cfg(unix)]`, because `script(1)` has no Windows
equivalent, so on Windows they compile to nothing. The behaviour itself is
the same on every platform, and `crossterm` implements it over ConPTY on
Windows.

A pty test must not depend on machine load. A fixed sleep passes on an idle
machine and fails while other worktrees build beside it. So
`opening_a_task_suspends_the_tui_and_runs_the_editor` and
`opening_a_task_still_works_with_hyperlinks_active` poll the transcript for
the frame they check.

= Catching compile errors early

#let rust-channel = fact("crate-map").at("rust-toolchain").at("channel")

#just.check-windows() compiles the workspace, tests included, for
`x86_64-pc-windows-gnu` from Linux, so a compile error shows up before you
push. It runs in its own dev shell (`nix develop .#windows`), because the
default shell's rustc has no Windows standard library. `pkgsCross.mingwW64`
supplies a rustc on the same pinned #rust-channel channel with the standard
library for both targets.

It is not a #just.check() arm, because it needs a separate toolchain and its
own `target/` tree. The Windows CI job still tests runtime behaviour and the
`-msvc` target.
