#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Cache cargo builds and the Nix closure in CI",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a Nix user gets a prebuilt mt from the cache],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[CI builds the flake on every push, not only at release],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 14),
)

== Problem

1. Read `.github/workflows/ci.yml`'s `check` job.
2. 🔴 It caches the Nix store but no cargo build, so every push compiles the workspace cold for clippy, the tests, the dev `mt` and the coverage rebuild.
3. 🔴 `release.yml`'s build and publish jobs save a rust-cache per tag, which no later release can read.
4. 🔴 CI never builds the flake, so a break in it shows first in the release hook's `just nix-build`.


== TO-BE

- 🟢 The `check` job restores compiled dependencies saved from `main`, keyed by `flake.lock` so a toolchain bump starts a fresh cache.
- 🟢 It substitutes from and pushes to the mlavrinenko Cachix cache, once the repo has a `CACHIX_AUTH_TOKEN` secret.
- 🟢 It runs `just nix-build` after `just check`.
- 🟢 The release jobs save no cache.


== Changes

- `.github/workflows/ci.yml`: cachix-action, rust-cache and a `Nix build` step in `check`.
- `.github/workflows/release.yml`: drop rust-cache from `build` and `publish`.


== Rollback

- Revert the commit.


== Out of scope

- `release.yml`'s Trusted Publishing and action bumps, postponed past 0.3.0.
- A flake `nixConfig` substituter. It prints an untrusted-setting warning on every local `nix develop` here, and the host already lists the cache.


== Dogfood

Every push to this repo after 0.3.0 runs it.
