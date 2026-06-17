#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Adopt gix as the git substrate (drop shell-out git)",
  status: done(2026, 6, 17),
  priority: "normal",
  estimate-hours: 8,
  relates-to: (
    link("swhid-mint-crate.typ"),
    link("mindtape-cleanup.typ"),
    link("converge-speconaut.typ"),
  ),
  tags: ("substrate",),
)

== Summary

Replace the hand-rolled `git -C <root> … capture` shell-out idiom with the
`gitoxide` (`gix`) crate, which gives a real git domain model instead of parsing
porcelain text. Three copies of the idiom exist today: `crates/mindtape-cli/src/
git.rs` (`toplevel`, `is_clean`, `capture`), `crates/swhid-mint/src/git.rs` (mint
+ resolve), and speconaut's `src/git.rs`. gitoxide's README points applications
to the high-level `gix` crate as the entrypoint; reach for the lower plumbing
crates (`gix-object`, `gix-hash`, `gix-revision`/`gix-revwalk`, `gix-status`) only
where `gix` does not yet cover a need.

== Scope

- `crates/swhid-mint`: the `git` feature currently shells out to compute the
  last-touch commit and verify the working blob equals the blob at that commit.
  sha1_git == git's own object id, so `gix-object`/`gix-hash` can compute ids
  natively and `gix-revwalk` finds the last-touch commit — no subprocess, no
  porcelain parsing. Keep the cargo-feature seam (`git` default-on, core works
  with `default-features = false`); the backend becomes gix internally.
- `crates/mindtape-cli/src/git.rs`: `toplevel` -> `gix::discover`; `is_clean` ->
  `gix` worktree status (`gix-status`). Delete the `Command::new("git")` plumbing.
- This crate is consumer-blind; speconaut adopts gix in its own `src/git.rs`
  (staged-files / index / recent-commit ops stay speconaut-side) under a separate
  speconaut task — not in scope here.

== Risks

- Flake build: gix pulls a deep dependency tree. Confirm the vendoring path
  (naersk/crate2nix or whatever the flake uses) resolves it offline before
  committing the dep — crates.io UA 403s bite `fetch-cargo-vendor` (see the global
  agent note). Validate `just check` builds clean in the dev shell.
- gix worktree status semantics vs `git status --porcelain` for the dirty-tree
  guard: keep the guard conservative (any tracked change refuses), match current
  behaviour, and cover with the existing `is_clean` regression test.

== Acceptance

No `Command::new("git")` remains in `crates/mindtape-cli` or `crates/swhid-mint`;
git access routes through `gix`; `swhid-mint`'s mint/resolve and the cleanup
dirty-tree guard keep their current behaviour under the existing tests; `just
check` passes in the flake dev shell.
