#!/usr/bin/env bash
# www/generate-release-targets.sh — derive www/generated/release-targets.json
# from .github/workflows/release.yml, for the install page.
#
# Split out of www/generate.sh (which calls this, still inside the lock it
# already holds) to bring that script back under .config/linecop.yaml's
# 300-line Shell cap. This one takes no lock of its own: it is called from inside the
# same critical section that `rm -rf`s and restores www/generated/, and writes
# exactly one artifact into the directory the caller hands it.
#
# Why derived rather than transcribed: the install page must never
# hand-transcribe what a release ships — the build-matrix `target:` list (one
# prebuilt binary per entry, attached to the GitHub Release) and the ordered
# `cargo publish -p <crate>` steps (crates.io). `yq` reads the workflow YAML
# as jq-filterable JSON (it's a jq wrapper, so the filters below are plain jq
# syntax); everything downstream is jq, matching generate.sh's other JSON
# captures (cli.json).
set -eo pipefail

REPO="$1"
OUT="$2"
if [ -z "$REPO" ] || [ -z "$OUT" ]; then
  echo "usage: generate-release-targets.sh <repo-root> <generated-dir>" >&2
  exit 1
fi

RELEASE_WF="$REPO/.github/workflows/release.yml"

# The upload-artifact "name:" template, e.g.
# "mindtape-${{ matrix.target }}${{ matrix.ext }}". Captured verbatim
# (placeholders and all) so the page derives each release's download filename by
# substitution instead of assuming a prefix — or, worse, assuming every target
# has no extension, which stopped being true when Windows joined the matrix.
ARTIFACT_TEMPLATE="$(yq -r \
  '.jobs.build.steps[] | select(.uses != null and (.uses | test("upload-artifact"))) | .with.name' \
  "$RELEASE_WF")"

# One entry per build-matrix target, in matrix order: the triple, plus the
# artifact name with that entry's own `target`/`ext` substituted in. Resolved
# here rather than in the page because only this side can read the matrix; the
# page would have to re-implement the substitution against fields it cannot see.
# Literal (non-regex) replacement, so the `${{ }}` braces need no escaping.
targets_json="[]"
while IFS=$'\t' read -r target ext; do
  [ -z "$target" ] && continue
  artifact="${ARTIFACT_TEMPLATE//\$\{\{ matrix.target \}\}/$target}"
  artifact="${artifact//\$\{\{ matrix.ext \}\}/$ext}"
  targets_json="$(jq --arg target "$target" --arg artifact "$artifact" \
    '. + [{target: $target, artifact: $artifact}]' <<<"$targets_json")"
done < <(yq -r \
  '.jobs.build.strategy.matrix.include[] | [.target, (.ext // "")] | @tsv' \
  "$RELEASE_WF")

# Published crates, in the workflow's publish order (`cargo publish -p X`
# lines only — the interstitial `sleep 30` steps have no `-p`).
PUBLISH_CRATES="$(yq -r '.jobs.publish.steps[].run // empty' "$RELEASE_WF" \
  | grep -oP '(?<=cargo publish -p )\S+')"

# A published crate is the installable CLI iff its own manifest declares a
# `[[bin]]` — checked against the crate's manifest in this workspace (a crate
# published from elsewhere, e.g. an extracted substrate crate that is not in
# crates/, can't be inspected here and is never mistaken for the CLI).
publish_json="[]"
while IFS= read -r crate; do
  [ -z "$crate" ] && continue
  is_bin=false
  manifest="$REPO/crates/$crate/Cargo.toml"
  if [ -f "$manifest" ] && grep -q '^\[\[bin\]\]' "$manifest"; then
    is_bin=true
  fi
  publish_json="$(jq --arg crate "$crate" --argjson bin "$is_bin" \
    '. + [{crate: $crate, bin: $bin}]' <<<"$publish_json")"
done <<<"$PUBLISH_CRATES"

jq -n \
  --argjson targets "$targets_json" \
  --argjson publish "$publish_json" \
  '{targets: $targets, publish: $publish}' \
  >"$OUT/release-targets.json"
