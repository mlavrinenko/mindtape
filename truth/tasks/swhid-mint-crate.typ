#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "swhid-mint: mint and resolve SWHIDs against a local git repo",
  status: done(2026, 6, 16),
  priority: "high",
  estimate-hours: 6,
  relates-to: (
    link("mindtape-cleanup.typ"),
    link("editor-tool-atop-harvest.typ"),
  ),
  tags: ("substrate",),
)

== Summary

Landed `crates/swhid-mint`: a consumer-blind crate bridging SWHID (ISO/IEC
18670:2025) and a local git repo. The core (`Swhid`, `render`, `parse`) is
VCS-agnostic; the `git` backend (default feature, shells out to the `git` binary)
mints and resolves. Knows nothing about tasks, tombstones, or any DSL.

- `git::mint_rev_path(root, path)` produces
  `swh:1:rev:<last-touch-commit>;path=<path>`, verifying the working blob matches
  the blob at that commit.
- `git::resolves(root, &id)` checks the commit exists and (if qualified) the path
  exists at it. Both `mindtape check` and speconaut `check` use this (replaces
  `check_git_refs`).
- `git::mint_content` exposes the `cnt` half of the scheme.

== Decisions (ADR 003)

- Hand-rolled format, not the `swhid` crate: for cnt/dir/rev the SWHID hash is
  sha1_git == git's own object SHA-1, so git computes it; we only prefix
  `swh:1:<kind>:`. Rationale is a comment in the crate.
- Backends are cargo features (`git` today, default-on); the core works with
  `default-features = false`. Other VCS slot in as features later.
- No origin, no blob in the identifier. Full 40-hex commit, never abbreviated.

== Acceptance (met)

Crate builds standalone with no task or domain knowledge; mint and resolve are
covered by 12 tests (including a temp git repo); `just check` passes. Consumption
by `mindtape cleanup` is tracked in that task.
