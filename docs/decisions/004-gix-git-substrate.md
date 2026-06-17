# 4. gix as the git substrate

Status: Accepted
Date: 2026-06-17

## Context

Three copies of the same idiom shelled out to the `git` binary and parsed its
porcelain text: `crates/swhid-mint/src/git.rs` (mint last-touch commit, verify
the working blob, resolve), `crates/mindtape-cli/src/git.rs` (`rev-parse
--show-toplevel`, `status --porcelain`), and speconaut's `src/git.rs`. Porcelain
parsing is brittle (locale, format drift, whitespace splitting) and pays a
process-spawn per call.

ADR 003 chose to shell out to `git` because for cnt/dir/rev the SWHID hash is
sha1_git — git already computes it — so no hashing library was needed and the
backend stayed dependency-free. That left the porcelain-parsing fragility in
place.

## Decision

1. Route all git access through `gix` (gitoxide), a pure-Rust git object model.
   No `Command::new("git")` in production code (test fixtures may still drive the
   real `git` binary to build repositories).

2. `swhid-mint`'s `git` feature backend becomes `gix` internally; the cargo
   feature seam is unchanged (`git` default-on; the core renders/parses with
   `default-features = false`). sha1_git == git's object id, so `gix` computes
   blob ids natively (`gix::objs::compute_hash`), walks ancestors for the
   last-touch commit (`Commit::ancestors`), and reads recorded blobs from trees
   (`Tree::lookup_entry_by_path`) — no subprocess, no porcelain.

3. `mindtape-cli`'s git module uses `gix::discover` for the work-tree root and
   `gix` worktree status (`gix-status` + `gix-dir`, untracked files included) for
   the dirty-tree guard, matching `git status --porcelain` breadth.

4. Feature minimization: both crates depend on `gix` with
   `default-features = false`. `swhid-mint` enables `revision` + `sha1`;
   `mindtape-cli` enables `status` + `sha1`. `sha1` is mandatory — `gix-hash`
   refuses to build without a hash backend.

5. `mindtape` (domain lib) keeps a `default-features = false` dep on `swhid-mint`
   for the `is_swhid_url` predicate only; it does not pull the `gix` tree.

6. speconaut adopts `gix` in its own `src/git.rs` (staged-files / index /
   recent-commit ops stay speconaut-side) under a separate speconaut task.

## Consequences

- Gain: a real git domain model instead of porcelain text; no process spawn per
  query; no locale/format fragility. Behaviour preserved under the existing
  `swhid-mint` mint/resolve tests and the `is_clean` / `toplevel` guards.
- Cost: `gix` is a deep dependency tree (~445 crates locked workspace-wide). The
  flake build vendors through `cargo` (naersk `buildPackage`), so the crates.io
  UA-403 that bites nixpkgs' `fetch-cargo-vendor` does not apply.
- ADR 003 point 2 is superseded on the backend mechanism only: the SWHID format,
  the hand-rolled parser, and the rejection of the `swhid` crate all stand. The
  reason for rejecting `swhid` (libgit2, archival-id focus, `clap` drag) is
  unaffected — `gix` is the substrate, not the SWHID library.

## Deprecation Criteria

Revisit if `gix`'s dependency weight becomes a build-time or vendoring problem
that the feature minimization cannot contain, or if a backend for a non-git VCS
is added (the feature seam already accommodates it). Supersede alongside ADR 003
when the shared crates move to their own repositories.
