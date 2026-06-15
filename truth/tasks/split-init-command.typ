#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Split init into install-lib and init",
  status: done(2026, 6, 16),
  priority: "normal",
  estimate-hours: 2,
  tags: ("cli",),
)

== Summary

`mindtape init` currently does two unrelated things: install the Typst library
(once per machine) and scaffold `.mindtape/config.toml` (once per project).
Coupling them forces users to pass `--no-lib` to avoid re-installing on every
new project.

== Decision

Split into two independent subcommands:

- `mindtape install-lib` — installs `@local/mindtape:<version>` into the
  user-local Typst packages dir. Idempotent; no `--force` needed since
  overwriting is always safe.
- `mindtape init` — scaffolds `.mindtape/config.toml` in the current directory.
  Keeps `--force` to allow overwriting.

Remove the `--no-lib` flag from `init`; it becomes a no-op concern.

== Scope

- Add `install-lib` subcommand (`install_lib` module or expand `init` module).
- Strip lib-install logic and `--no-lib` flag from `InitArgs` / `run`.
- Update CLI dispatch in `cli.rs`.
- Update tests: rename/add tests for the new subcommand; keep scaffold tests.
- Update `README.md` and any other docs that reference `init` flags.
