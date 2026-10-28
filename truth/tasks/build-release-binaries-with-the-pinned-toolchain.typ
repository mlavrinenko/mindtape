#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Build release binaries with the pinned toolchain",
  kind: "fix",
  priority: ice(
    impact: 9,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a release builds with the toolchain every gate ran],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 28),
)

== Problem

+ Push a `v*` tag, as `just release 0.3.0` did.
+ 🔴 `Build (aarch64-unknown-linux-gnu)` in `release.yml` run 40748057090 failed, and the other builds were cancelled:
  ```
  warn: the missing active toolchain `1.92.0-x86_64-unknown-linux-gnu` has been auto-installed
  error[E0463]: can't find crate for `core`
    = note: the `aarch64-unknown-linux-gnu` target may not be installed
  ```
  `dtolnay/rust-toolchain` added the target to `stable`, but cargo follows `rust-toolchain.toml`.

== Environment

- Tag `v0.3.0`, commit `fa6a464b`.
- GitHub Actions `ubuntu-latest`, rustup with `stable` 1.99.1 as its default.

== TO-BE

- 🟢 Every build in `release.yml` compiles with the toolchain `rust-toolchain.toml` pins, and each cross target is installed for it.
- 🟢 The Windows job in `ci.yml` installs the same toolchain, so its comment says what really runs.

== Changes

- `.github/workflows/release.yml`: the build and publish jobs run `rustup toolchain install`, and the build job adds the matrix target to it.
- `.github/workflows/ci.yml`: the Windows job installs the pinned toolchain the same way.

== Out of scope

- Moving the pin. 1.92.0 stays until a task of its own moves it.

== Dogfood

`gh workflow run release.yml -f tag=v0.3.0` finishes the 0.3.0 release with it.
