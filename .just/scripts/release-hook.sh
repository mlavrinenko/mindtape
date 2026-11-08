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
# NEW_VERSION, DRY_RUN and WORKSPACE_ROOT variables cargo-release sets. The
# release commit's subject comes from `pre-release-commit-message` in
# .config/release.toml, so the commit-msg hook judges it only after the push;
# this runs the same check before the dry run can call the release good.
set -euo pipefail
cd "$WORKSPACE_ROOT"
version="$NEW_VERSION"
tag="v$version"
publish_dir=""
refuse() {
    echo "error: $1" >&2; shift
    for h in "$@"; do echo "hint: $h" >&2; done
    if [ "$DRY_RUN" != true ] && [ -n "$(git status --porcelain)" ]; then
        echo "hint: git restore . drops the version bump; then re-run" >&2
    fi
    exit 1
}
for tool in git cargo cargo-audit curl gh jq; do
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
# A yanked crate keeps an existing Cargo.lock buildable but is refused to new
# installs, and `cargo publish --dry-run` only warns about it. `--deny yanked`
# would make the advisories this tree carries fatal too, so read the JSON report
# and fail on the yanked list alone.
audit_err=$(mktemp) \
    || refuse "can't make a temp file for the dependency audit" "check \$TMPDIR, then re-run"
trap 'rm -f "$audit_err"' EXIT
audit=$(cargo audit --json --quiet 2>"$audit_err") || true
if ! jq -e . >/dev/null 2>&1 <<<"$audit"; then
    cat "$audit_err" >&2
    refuse "cargo audit printed no JSON report" "check the network and the RustSec advisory database, then re-run"
fi
yanked=$(jq -r '.warnings.yanked // [] | length' <<<"$audit")
if [ "$yanked" -ne 0 ]; then
    jq -r '.warnings.yanked[] | "  \(.package.name) \(.package.version)"' <<<"$audit" >&2
    refuse "Cargo.lock pins $yanked yanked crate(s)" "run cargo update, then re-run"
fi
# The replacement dates [Unreleased] under the new version; a dry run, or a re-run
# after it, finds the entries under whichever heading holds them.
heading="Unreleased"
grep -Eq "^## \[${version//./\\.}\] - " CHANGELOG.md && heading="$version"
entries=$(sed -n "/^## \[${heading//./\\.}\]/,/^## \[/p" CHANGELOG.md | sed '1d;$d' | sed '/^[[:space:]]*$/d')
[ -n "$entries" ] \
    || refuse "CHANGELOG.md has nothing under [$heading] to release" "add the release's entries under '## [Unreleased]' and commit"
# The commit-msg hook would refuse the release subject after the push, so judge
# it here, where a dry run still stops. Run the mt the hook builds, not the dev
# shell's copy, which can be older than this tree.
msg=$(mktemp) \
    || refuse "can't make a temp file for the release commit message" "check \$TMPDIR, then re-run"
trap 'rm -f "$audit_err" "$msg"; [ -z "$publish_dir" ] || rm -rf "$publish_dir"' EXIT
subject=$(sed -n 's/^pre-release-commit-message = "\(.*\)"$/\1/p' .config/release.toml)
[ -n "$subject" ] \
    || refuse "can't read pre-release-commit-message from .config/release.toml" "restore the key, then re-run"
printf '%s\n' "${subject//\{\{version\}\}/$version}" >"$msg" \
    || refuse "can't write the release commit message to a temp file" "check \$TMPDIR, then re-run"
cargo run -q -p mindtape-cli --bin mt --target-dir target/mt-dev -- check --commit-msg "$msg" \
    || refuse "the release commit message breaks the commit rules" "fix what mt reports above, then re-run"
# The bump moves the version in the Cargo manifests, the sources of the
# `release-notes` and `cli-name` groups. The release commit is their review:
# it dates CHANGELOG.md, and the other dependents name the crate, not its
# version. A dry run sees the tree unbumped and records nothing.
if [ "$DRY_RUN" != true ]; then
    for group in release-notes cli-name; do
        just outdatty::update "$group" >/dev/null \
            || refuse "can't re-record outdatty group '$group'" "run just outdatty check and fix what it reports"
    done
fi
just check || refuse "just check failed on this tree" "fix what it reports above, then commit"
just nix-build || refuse "the flake doesn't build in a clean sandbox" "fix what just nix-build reports above"
# A dry run leaves the manifests at the old version, so a verifier would build
# the new CLI against the published `mindtape` at that old version, which may
# lack its new API. It verifies a throwaway copy bumped to $version instead.
publish_check() {
    local current target_dir
    current=$(sed -n 's/^version = "\([0-9][0-9.]*\)"/\1/p' Cargo.toml | head -1)
    [ -n "$current" ] \
        || refuse "can't read the workspace version" "restore Cargo.toml, then re-run"
    target_dir="$PWD/target"
    if [ "$current" = "$version" ]; then
        # --allow-dirty: on a real run the bump is not committed yet.
        cargo publish --dry-run --locked --allow-dirty --workspace --quiet
        return
    fi
    publish_dir=$(mktemp -d) \
        || refuse "can't make a temp dir for the publish check" "check \$TMPDIR, then re-run"
    git archive HEAD | tar -x -C "$publish_dir" \
        || refuse "can't copy HEAD for the publish check" "check git and \$TMPDIR, then re-run"
    sed -i "s/^version = \"$current\"/version = \"$version\"/" "$publish_dir/Cargo.toml" \
        || refuse "can't bump the copy's workspace version" "check sed, then re-run"
    sed -i "s/mindtape = { version = \"$current\", path = \"..\/mindtape\" }/mindtape = { version = \"$version\", path = \"..\/mindtape\" }/" \
        "$publish_dir/crates/mindtape-cli/Cargo.toml" \
        || refuse "can't bump the copy's mindtape dependency" "check sed, then re-run"
    (
        cd "$publish_dir"
        CARGO_TARGET_DIR="$target_dir" \
            cargo publish --dry-run --allow-dirty --workspace --quiet
    ) || refuse "cargo publish --dry-run failed on a copy bumped to $version" "fix the packaging error above"
}
publish_check
