// docs/src/contributing/release.typ renders docs/contributing/release.md
// through docs/generate-md.sh (typlite; docs/src/lib.typ lists the
// constraints).
//
// CONTRIBUTING.md's Version Management section holds the rules. This page
// holds what the release recipe runs, so that section stays short.
#import "../lib.typ": doc-title, just

#metadata((
  output: "docs/contributing/release.md",
)) <mindtape-md>

#doc-title[Releasing]

What #just.release("VERSION") runs, and how to finish a release that failed.
The tag push is the only publish path, so don't run `cargo publish` or push
`main` by hand.

= What the recipe runs

+ It runs #link("https://github.com/crate-ci/cargo-release")[cargo-release]
  (`.config/release.toml`), which refuses a dirty tree or a branch other than
  `main`, bumps the version, dates the entries under `[Unreleased]` and
  commits.
+ Between the bump and the commit, cargo-release runs
  `.just/scripts/release-hook.sh`. The hook re-records the outdatty groups
  whose sources are the Cargo manifests, since the release commit reviews the
  bump, holds the release commit's message to `mt check --commit-msg`, and
  refuses every other state a release has failed from.
+ The recipe pushes `main`, so CI runs on the release tree.
+ It tags only once that commit's `ci.yml` run is green.

`--dry-run` checks and pushes nothing. Its publish check verifies a throwaway
copy bumped to the new version, since the working tree still carries the old
one.

= Prelude versions

Before a release bumps the prelude, archive the outgoing current. Copy
`lib/prelude.typ` to
`crates/mindtape/src/loader/preludes/<old-version>/prelude.typ`, add a
`PreludeSnapshot` entry for `<old-version>` to `PRELUDE_SNAPSHOTS` and
`<old-version>` to `PRELUDE_SUPPORTED` in
`crates/mindtape/src/loader/embedded.rs`, and keep a fixture that loads it.
Then bump both `PRELUDE_VERSION` and `lib/typst.toml` to the new version. The
snapshot is immutable: a checksum test holds an archived copy, and a test
keeps the current snapshot equal to `lib/prelude.typ`.

A release drops a version on purpose by removing it from `PRELUDE_SUPPORTED`
and `PRELUDE_SNAPSHOTS` and deleting its snapshot and fixture. That release
stops reading the version everywhere, including a past commit in
`mt check --commits`.

= Build hosts

The Rust grammar behind an ast-grep query is C, compiled through `cc` at build
time, so every build host needs a C compiler, the Windows cross check included.
`ast-grep-language`'s default `builtin-parser` feature would pull every
grammar, so `crates/mindtape/Cargo.toml` keeps default features off and enables
one grammar crate per language. Measure the binary before adding another.

= After a failure

After a red CI, fix `main` and re-run the recipe: the bump and the dating stay
done.

A release whose workflow failed is finished on the same tag with
`gh workflow run release.yml -f tag=vVERSION`, which skips each crate
crates.io already has. A pushed tag never moves.
