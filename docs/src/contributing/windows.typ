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

What runs on Windows, what can't, and how you catch a Windows compile error
before CI does.

= What CI runs on Windows

The `windows` job in `.github/workflows/ci.yml` runs `cargo build` and
`cargo nextest run`. It doesn't run #just.check(), because the other gates
behave the same on every platform. Windows has no Nix, so
`taiki-e/install-action` installs nextest there. The job covers the code that
differs by platform: `cmd /C` against `sh -c` for an `[[on.flip]]` `run`, the
config, data and cache paths from `dirs`, and path separators.

On Windows, `dirs` reads the data path from a system API that no environment
variable moves. A test that sets `XDG_DATA_HOME` would test a path `mt` never
reads there. That's why `MINDTAPE_CONFIG_HOME`, `MINDTAPE_DATA_HOME` and
`MINDTAPE_CACHE_HOME` exist, and why `support::isolated_env` sets all three.
`tests/it/packs_cache_e2e.rs` fills the pack cache under the last one.
`tests/it/install_e2e.rs` sets them to check both where `mt install`
writes the `@local/mindtape` package and where loading a project finds it.
`tests/it/install_package_path_e2e.rs` leaves `MINDTAPE_DATA_HOME` unset and
sets `TYPST_PACKAGE_PATH` instead, which typst-world reads on every platform,
to check the same against Typst's own package directory.

= The pty tests are Unix-only

`tests/it/interactive_e2e.rs` drives `mt ls --interactive` under a real terminal
through `script(1)`, which the dev shell provides. It needs a terminal to
exercise the `guard` checks, `event_loop` and `draw`. It and
`tests/it/tty_e2e.rs` are `#![cfg(unix)]`, because `script(1)` has no Windows
equivalent, so on Windows they compile to nothing. The behavior is the same
on every platform, and `crossterm` implements it over ConPTY on Windows.

Don't let a pty test depend on machine load. A fixed sleep passes on an idle
machine and fails while other worktrees build beside it. So
`opening_a_task_suspends_the_tui_and_runs_the_editor` and
`opening_a_task_still_works_with_hyperlinks_active` poll the transcript for
the frame they check.

`tests/it/outdatty_lock_merge_e2e.rs` is `#![cfg(unix)]` too. It runs the
`.config/outdatty.lock` merge driver, a bash script that needs `yq`, `jq` and
`b3sum` from the dev shell, and the Windows job has no dev shell.

= Catching compile errors early

#let rust-channel = fact("crate-map").at("rust-toolchain").at("channel")

Run #just.check-windows() before you push. It compiles the workspace, tests
included, for `x86_64-pc-windows-gnu` from Linux. It runs in its own dev
shell (`nix develop .#windows`), because the default shell's rustc has no
Windows standard library. `pkgsCross.mingwW64` supplies a rustc on the same
pinned #rust-channel channel with the standard library for both targets.

It isn't a #just.check() arm, because it needs a separate toolchain and its
own `target/` tree. The Windows CI job still tests runtime behavior and the
`-msvc` target.
