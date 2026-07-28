#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Cross-check Windows from the dev shell",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 5),
  status: done(
    2026,
    7,
    28,
  )[flake.nix devShells.windows + just check-windows; the cfg(windows) coverage proven with a canary typo],
)

== Summary

Shipping a Windows binary put platform-conditional arms in the tree that nothing on a Linux dev machine compiles, so the only thing between a broken arm and a red CI run is remembering to look. Verifying that work took five ad-hoc nix shell invocations of a hand-assembled cross toolchain, none of it recorded anywhere a second person could rerun. rustup is the obvious route and is not available: it is silently broken on the NixOS host, and a rustup-managed target would drift from the rust-toolchain.toml pin anyway.

== Scope

- Add a windows dev shell to flake.nix carrying a rustc with std for the Windows GNU target and the Linux host both.
- Add a check-windows just recipe that cross-checks the workspace, tests included, in that shell.
- Keep it out of just check: a second toolchain closure and target tree is not a per-commit cost, and Windows CI already covers the regression.
- Document the recipe, and why the cross target is the GNU one while the release ships the MSVC one.
