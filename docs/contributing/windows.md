<!-- Generated from docs/src/contributing/windows.typ by `just docs md`. Do not edit; edit the source. -->

# Windows, paths and the pty tests

What runs on Windows, what can’t, how a path turns into text, and how you catch a Windows compile error before CI does.

## What CI runs on Windows

The `windows` job in `.github/workflows/ci.yml` runs `cargo build` and `cargo nextest run`. It doesn’t run `just check`, because the other gates behave the same on every platform. Windows has no Nix, so `taiki-e/install-action` installs nextest there. The job covers the code that differs by platform: `cmd /C` against `sh -c` for an `[[on.flip]]` `run`, the config, data and cache paths from `dirs`, and path separators. Windows runs rarely, so nextest runs there with `--no-fail-fast` and one red run lists every failure.

A test of a config command passes on Windows only if it’s portable. A string `run` must mean the same to `sh -c` and `cmd /C`, as the stub checkers in `tests/it/waits_e2e.rs` do. An array `run` needs no shell, so `tests/it/argv_run_e2e.rs` spawns `mt` itself, the one program every runner has. Mindtape’s own presets and packs ship gates with no `run` at all, and `no_builtin_declares_a_command`, `no_answer_set_declares_a_command` and `no_embedded_pack_declares_a_command` keep it that way.

The runner’s git checks text files out with `\r\n` line endings. A test that compares a committed file byte for byte with the `\n` its code writes fails there, though every line matches. So such a file gets `text eol=lf` in `.gitattributes`, as the `schema/*.json` goldens do.

On Windows, `dirs` reads the data path from a system API that no environment variable moves. A test that sets `XDG_DATA_HOME` would test a path `mt` never reads there. That’s why `MINDTAPE_CONFIG_HOME`, `MINDTAPE_DATA_HOME` and `MINDTAPE_CACHE_HOME` exist, and why `support::isolated_env` sets all three. `tests/it/packs_cache_e2e.rs` fills the pack cache under the last one. `tests/it/install_e2e.rs` sets them to check both where `mt install` writes the `@local/mindtape` package and where loading a project finds it. `tests/it/install_package_path_e2e.rs` leaves `MINDTAPE_DATA_HOME` unset and sets `TYPST_PACKAGE_PATH` instead, which typst-world reads on every platform, to check the same against Typst’s own package directory.

## Paths

A path shown to people uses the native separator, and a path a program reads uses `/`, as in git, rustc, ripgrep and fd. The CLI reference states the rule for users. In code, a path turns into text through one of the two helpers in `crates/mindtape/src/path_text.rs`:

- `mindtape::display_path` for a path a person reads: a message, a note, an error’s place, the `file` column. It honours `--path-separator`.
- `mindtape::slash_path` for a path a program reads or a file stores: the JSON and CSV `file` field, a link target, a lock entry.

`clippy.toml` refuses `Path::display`, so a new message can’t print a path any other way. A path that stays a path, such as a template directory the config passes on, keeps its own form through `to_string_lossy`.

A test that pins how a path reads passes `--path-separator` a `\`, so a Linux run takes the route Windows takes by default. A test about something else that prints a path wraps the expected path in `support::native`, which writes the platform’s separator.

## The pty tests are Unix-only

`tests/it/interactive_e2e.rs` drives `mt ls --interactive` under a real terminal through `script(1)`, which the dev shell provides. It needs a terminal to exercise the `guard` checks, `event_loop` and `draw`. It and `tests/it/tty_e2e.rs` are `#![cfg(unix)]`, because `script(1)` has no Windows equivalent, so on Windows they compile to nothing. The behavior is the same on every platform, and `crossterm` implements it over ConPTY on Windows.

Don’t let a pty test depend on machine load. A fixed sleep passes on an idle machine and fails while other worktrees build beside it. So `opening_a_task_suspends_the_tui_and_runs_the_editor` and `opening_a_task_still_works_with_hyperlinks_active` poll the transcript for the frame they check.

## Catching compile errors early

Run `just check-windows` before you push. It compiles the workspace, tests included, for `x86_64-pc-windows-gnu` from Linux. It runs in its own dev shell (`nix develop .#windows`), because the default shell’s rustc has no Windows standard library. `pkgsCross.mingwW64` supplies a rustc on the same pinned 1.92.0 channel with the standard library for both targets.

It isn’t a `just check` arm, because it needs a separate toolchain and its own `target/` tree. The Windows CI job still tests runtime behavior and the `-msvc` target.
