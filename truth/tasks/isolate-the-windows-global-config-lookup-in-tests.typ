#import "@local/mindtape:0.2.0": *

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
  status: wip(2026, 7, 29),
)

== Summary

On Windows dirs::config\_dir() resolves through the SHGetKnownFolderPath known-folder API, which no environment variable overrides. Two consequences. The e2e harness support::mt isolates the global defaults.toml layer by pointing XDG\_CONFIG\_HOME and HOME at an empty path, so on Windows it isolates nothing and the suite there still reads whatever the machine keeps in %APPDATA%/mindtape/defaults.toml. And install\_e2e asserts that mt install writes under the XDG\_CONFIG\_HOME and XDG\_DATA\_HOME it injects, which on Windows it will not, so those assertions should fail on the Windows CI job the moment that job actually runs. The tests/global\_config\_e2e.rs regression is cfg(unix) for the same reason.

== Scope

- Decide the injection seam: a MINDTAPE\_CONFIG\_HOME style env override read by the loader on every platform, or cfg(windows) test gating that admits the gap.
- Make support::mt isolate on Windows, and drop the cfg(unix) from tests/global\_config\_e2e.rs.
- Fix or gate install\_e2e, which asserts XDG paths that Windows ignores.
- Confirm against a real Windows CI run, not just just check-windows, which is compile-only.

== Decision

The env-override seam, not cfg(windows) gating: crates/mindtape/src/app\_dirs.rs owns both per-user directories behind MINDTAPE\_CONFIG\_HOME (global defaults.toml, config.schema.json, presets/, worktrees.toml) and MINDTAPE\_DATA\_HOME (the installed @local/mindtape package). A non-empty value replaces the platform lookup wholesale, for reading and for what mt install writes; set-but-empty means "no such directory", which drops the global layer rather than falling back to the machine's own; unset resolves through dirs as before. Gating would have left the Windows job running with the developer's — and the runner's — global config merged in, which is the actual hazard, not the failing assertion.

Every call site goes through the one seam: config\_layers (the xdg layer), install (defaults + schema), preset (user presets), worktrees (the registry), init (package\_dir). support::isolated\_env sets MINDTAPE\_CONFIG\_HOME alongside the XDG pair, which stays because HOME also caps the ancestor defaults.toml walk. global\_config\_e2e dropped its cfg(unix) and gained a third case: with XDG\_CONFIG\_HOME aimed at a directory that does hold a defaults.toml, an override elsewhere still hides it — the assertion the Unix-only seam would fail. install\_e2e injects the two MINDTAPE vars instead of the XDG ones, keeping the same asserted layout. The subprocess unit tests in worktrees.rs, init\_tests.rs and rm\_cli\_tests.rs switched too.

== Open

Windows CI runs cargo test --workspace, so the un-gated regression exercises the seam there; the run itself still has to be observed on the next push before this closes. just check and just check-windows are green locally.
