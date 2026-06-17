#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Stop the committed .cargo/config.toml leaking sccache into the nix build sandbox",
  status: done(2026, 6, 17),
  priority: "normal",
  estimate-hours: 1,
  relates-to: (
    link("adopt-gix-git-substrate.typ"),
  ),
  tags: ("tooling",),
)

== Summary

`nix build .#default` failed with `could not execute process sccache
…/rustc-wrapper-1.92.0/bin/rustc (never executed)`. Root cause: a git-tracked
`.cargo/config.toml` pinning `rustc-wrapper = "sccache"`. naersk filters the
flake source to git-tracked files and vendors that config into the sandboxed
package build, where sccache is absent and cargo fails trying to exec it. The
devShell `RUSTC_WRAPPER` was empty and `~/.cargo/config.toml` was empty — the
project-local committed config was the sole source. Surfaced during
adopt-gix-git-substrate (whose dev-shell acceptance was already met); the
sandbox break is a distinct concern. cratemplate already fixed this class
(`18c303c`); mindtape predated the fix.

== Scope

- Delete the committed `.cargo/config.toml`; gitignore `/.cargo/`.
- Move the wrapper to the devShell as `RUSTC_WRAPPER = "sccache"` (loaded via
  direnv), so sccache still wraps every dev/interactive build but never reaches
  the `nix build` sandbox.
- Gitignore the `result` symlink `nix build` produces.
- Add a `nix build` regression gate (stage first — flakes only see tracked
  files) so this package-build break class is caught.

== Acceptance

`.cargo/config.toml` is gone and gitignored; the devShell exports
`RUSTC_WRAPPER=sccache` with sccache resolving on PATH; `nix build .#default`
succeeds and produces `result/bin/mindtape`; a Justfile recipe runs the nix
build gate.
