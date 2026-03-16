# ADR-001: Pin Rust Toolchain Version

**Status**: Accepted
**Date**: 2026-03-16

## Context

CI used `dtolnay/rust-toolchain@stable` which always installs the latest stable
Rust release, while the local Nix devshell provided a specific Rust version from
`nixpkgs-unstable`. Different `rustfmt` versions apply different line-breaking
heuristics, so code formatted locally could fail CI's `cargo fmt --check` — or
vice versa. This caused spurious CI failures that were invisible during local
development.

Concretely, Rust 1.92.0's `rustfmt` wraps a chained method call onto two lines,
while a newer version keeps it on one line. Neither is wrong — they simply
disagree.

## Decision

1. **Pin the Rust channel in `rust-toolchain.toml`** to an exact version
   (e.g. `channel = "1.92.0"`) instead of `"stable"`. This file is the single
   source of truth for the project's Rust version.

2. **CI reads from `rust-toolchain.toml`** via `rustup show` instead of using
   `dtolnay/rust-toolchain@stable`. This ensures CI installs the same toolchain
   the file specifies.

3. **The Nix devshell includes `rustfmt`** explicitly (`flake.nix`). Previously
   it was missing — `rustc`, `cargo`, and `clippy` were listed but `rustfmt` was
   not, so `cargo fmt` silently fell back to whatever `rustfmt` the system or
   rustup provided.

## Consequences

- Local and CI formatting is guaranteed identical as long as both use the pinned
  version.
- Upgrading Rust becomes a deliberate act: bump `rust-toolchain.toml`, update
  the Nix flake's nixpkgs pin, run `just fmt-check`, and commit any reformatted
  files in the same PR.
- The Nix devshell may still provide a slightly different Rust version than the
  pinned one (Nix builds from nixpkgs, not rustup). If versions drift apart,
  re-pin nixpkgs or switch to a Rust Nix overlay (e.g. `fenix`,
  `rust-overlay`). For now the versions match.

## Deprecation Criteria

This ADR can be dropped if:

- The project adopts a Nix-based Rust overlay (`fenix` / `rust-overlay`) that
  reads `rust-toolchain.toml` directly — the CI step and manual pin become
  redundant since both environments derive from the same file.
- `rustfmt` stabilises its formatting output across versions so that pinning is
  no longer necessary (unlikely in practice).
