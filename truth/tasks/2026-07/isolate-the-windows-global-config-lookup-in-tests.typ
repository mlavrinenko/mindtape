#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Isolate the Windows global config lookup in tests",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (files: 4),
  status: done(
    2026,
    8,
    1,
  )[Requires a CI run, but we'll do it anyway.],
)

== Summary

On Windows dirs::config\_dir() resolves through the SHGetKnownFolderPath known-folder API, which no environment variable overrides. Two consequences. The e2e harness `support::mt` isolates the global defaults.toml layer by pointing `XDG_CONFIG_HOME` and `HOME` at an empty path. On Windows that isolates nothing, and the suite there still reads whatever the machine keeps in %APPDATA%/mindtape/defaults.toml. `install_e2e` asserts that mt install writes under the `XDG_CONFIG_HOME` and `XDG_DATA_HOME` it injects. On Windows it will not. Those assertions should fail on the Windows CI job the moment that job actually runs. The `tests/global_config_e2e.rs` regression is `cfg(unix)` for the same reason.

== Scope

- Decide the injection seam: a `MINDTAPE_CONFIG_HOME` style environment override read by the loader on every platform, or `cfg(windows)` test gating that admits the gap.
- Make support::mt isolate on Windows, and drop the cfg(unix) from tests/global\_config\_e2e.rs.
- Fix or gate install\_e2e, which asserts XDG paths that Windows ignores.
- Confirm against a real Windows CI run, not only the `just check-windows` recipe, which is compile-only.

== Decision

The env-override seam, not `cfg(windows)` gating. The file crates/mindtape/src/app\_dirs.rs owns both per-user directories. `MINDTAPE_CONFIG_HOME` covers global defaults.toml, config.schema.json, presets/ and worktrees.toml. `MINDTAPE_DATA_HOME` covers the installed @local/mindtape package. A non-empty value replaces the platform lookup wholesale, for reading and for what mt install writes. A set-but-empty value means "no such directory", which drops the global layer rather than falling back to the machine's own. An unset value resolves through dirs as before. Gating would have left the Windows job running with the developer's and the runner's global config merged in, which is the actual hazard, not the failing assertion.

Every call site goes through the one seam: config\_layers (the `xdg` layer), install (defaults + schema), preset (user presets), worktrees (the registry), init (package\_dir). `support::isolated_env` sets `MINDTAPE_CONFIG_HOME` alongside the `XDG` pair, which stays because `HOME` also caps the ancestor defaults.toml walk. `global_config_e2e` dropped its `cfg(unix)` and gained a third case: with `XDG_CONFIG_HOME` aimed at a directory that does hold a defaults.toml, an override elsewhere still hides it. That is the assertion the Unix-only seam would fail. `install_e2e` injects the two `MINDTAPE` variables instead of the `XDG` ones, keeping the same asserted layout. The subprocess unit tests in worktrees.rs, init\_tests.rs and rm\_cli\_tests.rs switched too.

== Open

Windows CI runs `cargo test --workspace`. The un-gated regression then exercises the seam there. Someone still has to observe the run on the next push before this closes. `just check` and `just check-windows` are green locally.
