#!/usr/bin/env bash
# .just/scripts/release-hook.sh — the `pre-release-hook` of .config/release.toml,
# which `just release VERSION` runs through cargo-release. cargo-release itself
# refuses a dirty tree and a branch other than main; this refuses the rest. Every
# refusal guards a way a release has gone wrong, and prints one `error:` and at
# most two `hint:` lines. v0.2.0's crates were published by hand before its tag
# was pushed, so every run of the tag's workflow failed at `cargo publish` and
# no GitHub release was made: a version already on crates.io is refused here,
# and the tag push stays the only publish path.
#
# cargo-release runs this after the bump and the changelog replacement, and also
# in a dry run (DRY_RUN=true), where it sees the tree unchanged. It reads the
# NEW_VERSION, DRY_RUN and WORKSPACE_ROOT variables cargo-release sets.
set -euo pipefail
cd "$WORKSPACE_ROOT"
version="$NEW_VERSION"
tag="v$version"
refuse() {
    echo "error: $1" >&2; shift
    for h in "$@"; do echo "hint: $h" >&2; done
    if [ "$DRY_RUN" != true ] && [ -n "$(git status --porcelain)" ]; then
        echo "hint: git restore . drops the version bump; then re-run" >&2
    fi
    exit 1
}
for tool in git cargo curl gh; do
    command -v "$tool" >/dev/null \
        || refuse "'$tool' is not on PATH" "run inside the dev shell"
done
gh auth status >/dev/null 2>&1 || refuse "gh is not logged in" "gh auth login"
git fetch --quiet origin main || refuse "can't fetch 'origin main'" "check the network and the origin remote"
git merge-base --is-ancestor origin/main HEAD \
    || refuse "origin/main has commits that main lacks" "git pull --ff-only origin main, then re-run"
if git rev-parse -q --verify "refs/tags/$tag" >/dev/null \
    || git ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null; then
    refuse "'$tag' already exists" "a pushed tag never moves; finish a failed release with: gh workflow run release.yml -f tag=$tag"
fi
repo_url=$(sed -n 's/^repository = "\(.*\)"/\1/p' Cargo.toml | head -1)
for crate in mindtape mindtape-cli; do
    status=$(curl -sS -o /dev/null -w '%{http_code}' -A "$repo_url" \
        "https://crates.io/api/v1/crates/$crate/$version" || true)
    case "$status" in
        404) ;;
        200) refuse "'$crate $version' is already on crates.io" "a published version is never re-published; bump to the next version" ;;
        *) refuse "crates.io answered HTTP '${status:-nothing}' for '$crate $version'" "retry once crates.io is reachable" ;;
    esac
done
# The replacement dates [Unreleased] under the new version; a dry run, or a re-run
# after it, finds the entries under whichever heading holds them.
heading="Unreleased"
grep -Eq "^## \[${version//./\\.}\] - " CHANGELOG.md && heading="$version"
entries=$(sed -n "/^## \[${heading//./\\.}\]/,/^## \[/p" CHANGELOG.md | sed '1d;$d' | sed '/^[[:space:]]*$/d')
[ -n "$entries" ] \
    || refuse "CHANGELOG.md has nothing under [$heading] to release" "add the release's entries under '## [Unreleased]' and commit"
just check || refuse "just check failed on this tree" "fix what it reports above, then commit"
just nix-build || refuse "the flake doesn't build in a clean sandbox" "fix what just nix-build reports above"
# --allow-dirty: on a real run the bump is not committed yet.
cargo publish --dry-run --locked --allow-dirty --workspace --quiet \
    || refuse "cargo publish --dry-run failed" "fix the packaging error above"
