#!/usr/bin/env bash
# Body of `just release VERSION`; DRY_RUN comes from the recipe's --dry-run.
# cargo-release (release.toml) bumps, dates CHANGELOG.md, runs release-hook.sh,
# commits and pushes main. Then this waits for ci.yml on that commit and tags
# only if it is green. A red run leaves main pushed and no tag, and a re-run
# repeats nothing the first run did. A tag whose workflow failed is re-run,
# never re-tagged: `gh workflow run release.yml -f tag=v<version>`.
set -eo pipefail
version="$1"
refuse() { echo "error: $1" >&2; shift; for h in "$@"; do echo "hint: $h" >&2; done; exit 1; }
tag=v$version
args=(-p mindtape-cli --no-confirm)
if [ -z "$DRY_RUN" ]; then
    read -r -p "Push main and $tag to origin, proceed? [y/N] " answer
    [ "$answer" = y ] || exit 1
    args+=(--execute)
fi
cargo release "$version" "${args[@]}" --no-tag
if [ -n "$DRY_RUN" ]; then
    echo "dry run: every check passed"
    echo "would push main, wait for ci.yml on it, then tag and push '$tag'"
    exit 0
fi
head=$(git rev-parse HEAD)
short=$(git rev-parse --short HEAD)
run=""
deadline=$((SECONDS + 300))
while :; do
    run=$(gh run list --commit "$head" --workflow ci.yml --limit 1 \
        --json databaseId --jq '.[0].databaseId // empty') \
        || refuse "can't list ci.yml runs for '$short'" "check gh auth status"
    [ -n "$run" ] && break
    [ "$SECONDS" -lt "$deadline" ] \
        || refuse "no ci.yml run showed up for '$short' in 5 minutes, so '$tag' was not made" "main is pushed; once ci.yml is green on it, re-run just release $version"
    sleep 5
done
echo "waiting for ci.yml run '$run' on '$short'"
if ! gh run watch "$run" --exit-status --compact --interval 15 >/dev/null; then
    conclusion=$(gh run view "$run" --json conclusion --jq .conclusion 2>/dev/null || true)
    refuse "ci.yml ended '${conclusion:-unknown}' on '$short', so '$tag' was not made" \
        "see why: gh run view $run --log-failed" \
        "fix main and re-run just release $version, or retry a flake with: gh run rerun $run --failed"
fi
cargo release tag "${args[@]}"
cargo release push "${args[@]}" \
    || { git tag -d "$tag" >/dev/null; refuse "origin rejected the push of '$tag'" "re-run just release $version"; }
echo "pushed '$tag'; follow it with: gh run list --workflow release.yml --limit 1"
