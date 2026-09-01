#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "verify the Windows nextest job on GitHub",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 2,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (files: 1),
  assignee: "max",
  links: related(
    "adopt-cargo-nextest-as-the-test-runner.typ",
  )[the migration this verifies — the Windows job went from `cargo test` to `cargo nextest run`],
  status: proposed(2026, 9, 1),
)

== Summary

The Windows CI job (`ci.yml`, job `windows`) runs the suite on a bare rustup
toolchain with no Nix. It now installs the same runner the Linux gate uses —
`taiki-e/install-action` at the pinned SHA, tool `nextest` — and runs `cargo
nextest run --workspace`. The `mt`-invoking e2e tests (`install_e2e.rs`, the
`cmd /C` vs `sh -c` spawn seams, the `dirs` data-path tests) are exactly the
ones nobody can prove green from Linux. Nothing local exercises a Windows
runner; only a pushed run answers that.

== Verification

Push main and watch job `windows` on `.github/workflows/ci.yml`: the `Test`
step must run 100% of the workspace's non-`cfg(unix)` tests through nextest.
The setup step's `install-action` fetch is the one new failure mode. Blockers:
Windows runner unreachable by the repo's pin, or the prebuilt binary not
building against the pinned 2137-test suite. There is no Linux-side substitute
for this run.
