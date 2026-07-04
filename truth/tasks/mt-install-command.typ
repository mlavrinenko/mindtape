#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "`mt install` — set up the user environment (lib + XDG defaults)",
  status: wip(2026, 7, 4),
  priority: ice(impact: 6, confidence: 0.8, ease: 6),
  tags: ("cli",),
  depends-on: (
    link("config-embedded-defaults.typ"),
    link("config-defaults-discovery.typ"),
  ),
)

== Summary

`mt init` scaffolds a PROJECT (`.mindtape/config.toml`). `mt install` sets up the
USER ENVIRONMENT: the `@local/mindtape` Typst package (what `mt install-lib` does
today) plus the XDG global `defaults.toml` (the app-level config layer). Two
distinct scopes, two commands.

== Scope

- New `mt install` subcommand (clap `Install(InstallArgs)` in
  `crates/mindtape-cli/src/cli.rs:31`), dispatching to `init.rs` (or a new
  `install.rs`). By default performs BOTH installs:
  - Lib: reuse `install_lib()` (`init.rs:36`) — `@local/mindtape` into the Typst
    local-packages dir.
  - Defaults: write `dirs::config_dir()/mindtape/defaults.toml`, seeded from the
    embedded floor. Seed with a COMMENTED copy (documented starter the user can
    trim), not the bare values — so the global layer is editable and discoverable.
- Flags:
  - `--no-lib` — skip the Typst package install.
  - `--no-defaults` — skip the XDG `defaults.toml` install.
  - `--force` — overwrite an existing XDG `defaults.toml` (default: leave it,
    print "already exists", mirroring `scaffold_config` at `init.rs:64`).
- Reconcile `install-lib`: keep `mt install-lib` as a hidden deprecated alias that
  runs the lib-only path (equivalent to `mt install --no-defaults`), so existing
  docs/muscle-memory keep working; note the deprecation in its help.
- Docs to update in the SAME change (flag-coupled, avoids drift gate):
  `README.md`, `www/index.html`, and the `[typst]` comment in
  `crates/mindtape-cli/src/default-config.toml` that references `mt install-lib`.

== Tests

- `mt install` into a tempdir HOME (override `HOME`/XDG env in the test) writes
  both the package files and `.../mindtape/defaults.toml`.
- `--no-lib` writes only defaults; `--no-defaults` writes only the lib.
- Existing XDG defaults are preserved without `--force`, overwritten with it.
- The seeded `defaults.toml` parses through `parse_config` (commented starter is
  valid TOML and merges to the floor).

== Notes

- Idempotent and safe to re-run; only `--force` overwrites the user's edited
  global config.
- The seeded defaults content is the same floor `mt config show --defaults`
  prints; keep them sourced from one embedded string to avoid a third copy.
