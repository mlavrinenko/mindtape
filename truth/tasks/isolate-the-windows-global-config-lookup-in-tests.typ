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
  status: proposed(2026, 7, 28),
)

== Summary

On Windows dirs::config\_dir() resolves through the SHGetKnownFolderPath known-folder API, which no environment variable overrides. Two consequences. The e2e harness support::mt isolates the global defaults.toml layer by pointing XDG\_CONFIG\_HOME and HOME at an empty path, so on Windows it isolates nothing and the suite there still reads whatever the machine keeps in %APPDATA%/mindtape/defaults.toml. And install\_e2e asserts that mt install writes under the XDG\_CONFIG\_HOME and XDG\_DATA\_HOME it injects, which on Windows it will not, so those assertions should fail on the Windows CI job the moment that job actually runs. The tests/global\_config\_e2e.rs regression is cfg(unix) for the same reason.

== Scope

- Decide the injection seam: a MINDTAPE\_CONFIG\_HOME style env override read by the loader on every platform, or cfg(windows) test gating that admits the gap.
- Make support::mt isolate on Windows, and drop the cfg(unix) from tests/global\_config\_e2e.rs.
- Fix or gate install\_e2e, which asserts XDG paths that Windows ignores.
- Confirm against a real Windows CI run, not just just check-windows, which is compile-only.
