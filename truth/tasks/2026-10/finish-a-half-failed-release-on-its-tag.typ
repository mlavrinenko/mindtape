#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Refuse a release tag that cannot finish, and finish one on its tag",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a release that cannot finish fails before its tag exists, and no tag lands on a red commit.],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a re-run finishes a failed release run, not a new version.],
      docs-match-code: helps[CONTRIBUTING states what `just release` refuses and how to re-run a tag],
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    7,
  )[release-dry-run refuses before tagging. The workflow re-runs a tag and skips published crates.],
)

== AS-IS

+ A maintainer published `mindtape` 0.2.0 to crates.io by hand before pushing `v0.2.0`.
+ 🔴 The tag's four release runs all failed at `cargo publish`, so `v0.2.0` has no GitHub release, and `mindtape-cli` has never reached crates.io.
+ 🔴 `release.yml` runs only on a tag push, so nothing can finish that release on the same tag. outdatty v0.6.0 and two typst crates hit the same wall.
+ 🔴 `just release` tags without checking that crates.io lacks the version, that the tag is new, or that CI passed on the commit.
+ 🔴 A guard that only refuses until `main` matches `origin/main` with green CI leaves three steps by hand. They are push `main`, wait for CI, and run the recipe. The push is the step that gets forgotten, as `push-main-from-just-release` found with GitHub `main` hundreds of commits behind. A wait judged by eye can tag a commit whose CI then goes red.


== TO-BE

- 🟢 `release.yml` skips `cargo publish` for each crate whose version crates.io already has, and still creates the GitHub release.
- 🟢 `gh workflow run release.yml -f tag=vX.Y.Z` re-runs an existing tag, every job checking out that tag. A re-released old tag doesn't take "Latest".
- 🟢 `just release X.Y.Z` refuses a dirty tree, a branch other than main, an `origin/main` with commits `main` lacks, and an existing tag. It also refuses a version on crates.io, no dated CHANGELOG section, a failing gate or `cargo publish --dry-run`, with one `error:` line and at most two `hint:` lines.
- 🟢 Then it pushes `main` fast-forward only and waits for the `ci.yml` run on that exact commit. It tags and pushes the tag only if that run is green, so one command cannot forget the push or tag a red commit. A red run leaves `main` pushed and no tag.
- 🟢 `just release-dry-run X.Y.Z` runs the checks and says what it would push, pushing nothing.


== Changes

- `.github/workflows/release.yml`: `workflow_dispatch` with a `tag` input, one run per tag, a meta job, idempotent publish per crate, an explicit release tag.
- `.just/scripts/release.sh`: the preflight, the push of `main`, the wait on its `ci.yml` run and the tag, and a `--dry-run` mode the confirm handshake does not gate.
- `Justfile`: `release-dry-run`, and the release doc comment.
- `docs/src/contributing.typ`, `CONTRIBUTING.md`: the release rules.


== Rollback

- Revert the commit. Neither file format nor config key changes.


== Out of scope

- Repairing `v0.2.0`: its tree has another crate layout and a nixpkgs pin from before the crates.io download fix, so this workflow's check job cannot pass on it.
- Releasing 0.3.0.


== Dogfood

The next release of this repo goes through `just release-dry-run` and `just release`.

