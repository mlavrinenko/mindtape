#!/usr/bin/env bash
# .just/scripts/release.sh — the body of the `release` and `release-dry-run`
# recipes, extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipes that invoke it; the prose naming
# what it refuses to release over stays above `release` in the Justfile.
# Takes the version as $1, and `--dry-run` as $2.
#
# The `[confirm(...)]` guard stays on the recipe, where `just` enforces it —
# this script cannot re-implement it, and must not let anyone step around it.
# So a real run refuses unless MINDTAPE_RELEASE_CONFIRMED is set, which only the
# recipe sets, right after the prompt is answered. Same handshake shape as
# MINDTAPE_GENERATED_LOCK. Without it, extracting the body would have turned a
# confirmed, tag-pushing recipe into a script anyone can run straight, and the
# `git push origin` at the bottom is not undoable. `--dry-run` pushes nothing,
# so it needs no confirmation.
#
# Every refusal guards a way a release has gone wrong, and prints one `error:`
# and one `hint:` line. v0.2.0's crates were published by hand before its tag
# was pushed, so every run of the tag's workflow failed at `cargo publish` and
# no GitHub release was made: a version already on crates.io is refused here,
# and the tag push stays the only publish path.
set -euo pipefail
refuse() { echo "error: $1" >&2; echo "hint: $2" >&2; exit 1; }
version="${1:-}"
dry_run=false
case "${2:-}" in
    "") ;;
    --dry-run) dry_run=true ;;
    *) refuse "unknown flag $2" "just release-dry-run VERSION, then just release VERSION" ;;
esac
if ! $dry_run && [ -z "${MINDTAPE_RELEASE_CONFIRMED:-}" ]; then
    refuse "release.sh tags and pushes only behind the recipe's confirmation" "run just release $version"
fi
tag="v$version"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] \
    || refuse "'$version' is not plain semver" "give it without the v: just release 0.3.0"
cargo_version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
[ "$version" = "$cargo_version" ] \
    || refuse "requested $tag but Cargo.toml is $cargo_version" "bump [workspace.package] version and mindtape-cli's mindtape dependency to $version, and commit"
for tool in git cargo curl gh; do
    command -v "$tool" >/dev/null \
        || refuse "$tool is not on PATH" "run inside the dev shell, with gh authenticated (gh auth login)"
done
[ -z "$(git status --porcelain)" ] \
    || refuse "the working tree has uncommitted changes" "commit or discard them, then re-run"
branch=$(git symbolic-ref --short -q HEAD || true)
[ "$branch" = main ] || refuse "HEAD is on '${branch:-a detached commit}', not main" "git switch main"
git fetch --quiet origin main || refuse "cannot fetch origin main" "check the network and the origin remote"
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] \
    || refuse "main and origin/main differ, so the tag would name a commit CI never saw" "git push origin main (pages.yml deploys the site from it), then wait for ci.yml"
if git rev-parse -q --verify "refs/tags/$tag" >/dev/null \
    || git ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null; then
    refuse "$tag already exists" "a pushed tag never moves; finish a failed release with: gh workflow run release.yml -f tag=$tag"
fi
for crate in mindtape mindtape-cli; do
    status=$(curl -sS -o /dev/null -w '%{http_code}' -A "mindtape release preflight" \
        "https://crates.io/api/v1/crates/$crate/$version" || true)
    case "$status" in
        404) ;;
        200) refuse "$crate $version is already on crates.io" "a published version is never re-published; bump to the next version" ;;
        *) refuse "crates.io answered HTTP ${status:-nothing} for $crate $version" "retry once crates.io is reachable" ;;
    esac
done
unreleased=$(sed -n '/^## \[Unreleased\]/,/^## \[/p' CHANGELOG.md | sed '1d;$d' | sed '/^[[:space:]]*$/d')
[ -z "$unreleased" ] \
    || refuse "CHANGELOG.md has a non-empty [Unreleased] section" "move it under '## [$version] - $(date +%F)' and commit"
grep -Eq "^## \[${version//./\\.}\] - [0-9]{4}-[0-9]{2}-[0-9]{2}" CHANGELOG.md \
    || refuse "CHANGELOG.md has no dated section for $version" "add '## [$version] - $(date +%F)' with the release's entries and commit"
ci=$(gh run list --commit "$(git rev-parse HEAD)" --workflow ci.yml --limit 1 \
    --json status,conclusion --jq '.[] | "\(.status) \(.conclusion)"') \
    || refuse "cannot list CI runs for HEAD" "check gh auth status"
case "$ci" in
    "completed success") ;;
    "") refuse "CI has not run on HEAD" "push main and wait for ci.yml to pass" ;;
    completed*) refuse "CI did not pass on HEAD (${ci#completed })" "fix main until ci.yml is green: gh run list --workflow ci.yml" ;;
    *) refuse "CI is still running on HEAD" "wait for it: gh run watch" ;;
esac
just check || refuse "just check failed on this tree" "fix what it reports above, commit and push"
just nix-build || refuse "the flake does not build in a clean sandbox" "fix what just nix-build reports above"
cargo publish --dry-run --locked --workspace --quiet \
    || refuse "cargo publish --dry-run failed" "fix the packaging error above"
if $dry_run; then
    echo "dry run: every check passed; $tag is ready for just release $version"
    exit 0
fi
git tag -a "$tag" -m "$tag"
git push origin "$tag"
echo "pushed $tag; follow it with: gh run list --workflow release.yml --limit 1"
