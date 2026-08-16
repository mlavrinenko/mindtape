<!-- Generated from docs/src/contributing/windows.typ by `just docs-md`. Do not edit; edit the source. -->

# Windows and the pty tests

Deep dive behind two lines of CONTRIBUTING.md’s Testing section: what runs on Windows, what cannot, and how to catch a break before CI does.

## What CI runs on Windows

CI runs the suite on Windows (`.github/workflows/ci.yml`, job `windows`), which is where the platform seams get exercised — `cmd /C` vs `sh -c` for `[[on.flip]]` `run`, the `dirs` config/data paths, path separators. It runs plain `cargo build` / `cargo test`, not `just check`: there is no Nix there, and the remaining gates are platform-independent.

## The pty tests are Unix-only

`tests/interactive_e2e.rs` drives `mt ls --interactive` under a real PTY via `script(1)` (util-linux, in the flake devShell) — it needs a TTY to exercise the `guard` checks and `event_loop`/`draw`. It and `tests/tty_e2e.rs` are `#![cfg(unix)]`: `script(1)` has no Windows equivalent, so on Windows they compile to nothing rather than being skipped at runtime.

What is unverified on Windows there is the harness, not the feature. The behaviour is platform-independent and `crossterm` implements it over ConPTY.

## Catching compile breakage early

`just check-windows` cross-checks the workspace (tests included) against `x86_64-pc-windows-gnu` from Linux, so compile breakage surfaces before the push rather than in CI.

It runs in its own dev shell (`nix develop .#windows`) because the default shell’s rustc carries no Windows std. `pkgsCross.mingwW64` supplies a rustc — the same pinned 1.92.0 — that has std for the target and the host both, so no rustup and no second toolchain manager is involved.

It is deliberately not a `just check` arm: it pulls a separate toolchain closure and `target/` tree. Windows CI still owns runtime behaviour and the `-msvc` target; this only moves the compile half earlier.
