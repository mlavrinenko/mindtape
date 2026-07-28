#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Ship Windows binaries",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 3,
  ),
  difficulty: 5,
  tags: ("tooling",),
  estimate: (files: 4),
  status: proposed(2026, 7, 28),
)

== Summary

release.yml builds four targets, all Linux and macOS, so there is no Windows binary and no evidence the tool works there at all. Nothing in the codebase is inherently POSIX-only, but several surfaces are untested on Windows: the XDG paths mt install writes, the Typst local-package directory, editor resolution in mt go and mt add --edit, the PTY-driven interactive test, and path separators in the derived-tag and link-resolution code. Excluding Windows silently costs a large share of the potential audience and invites the question on any public launch.

== Scope

- Add x86\_64-pc-windows-msvc to the release matrix and to the docs-derived install target list.
- Audit the platform-specific surfaces: config and data dirs, the Typst package path, editor launch, path separator handling in derived tags and links.
- Get CI running the test suite on Windows; decide what the PTY-based interactive test does there.
- State the support level in the README once it is known, rather than leaving it unsaid.
