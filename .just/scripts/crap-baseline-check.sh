#!/usr/bin/env bash
# .just/scripts/crap-baseline-check.sh: the body of the `crap::baseline-check`
# recipe (truth/tasks/relativize-the-crap-baseline-s-committed-file-paths.typ).
# Runs from the repo root, like the recipe that invokes it. The mint script
# (.just/scripts/crap-baseline.sh) calls it a second time, and a bad mint fails
# at the mint rather than at the next `just check`.
#
# What it locks. `cargo crap --workspace` reports the member roots `cargo metadata` gives it,
# which are absolute, and cargo-crap 0.3.1 has no flag to shorten them. crap-baseline.sh strips
# the workspace root back off before writing the tracked file. The mint succeeding doesn't
# enforce that, and a re-mint by hand (`cargo crap … --output .config/crap-baseline.json`, the
# exact command the recipe ran before) puts every local path straight back.
# The leak is invisible in review (a 1000-entry JSON diff nobody reads line by line), so it
# needs a check rather than a convention.
#
# Why it is not a gitleaks rule, measured under `--log-level trace` on gitleaks
# 8.30.1. The default ruleset's global allowlist matches `/home/<user>/` and
# drops any custom rule's finding on it (`skipping finding: global allowlist
# allowed-regex=true`), and an extending config can only add allowlists, never
# remove one. A machine-local-path rule is dead the moment a config sets
# `[extend] useDefault = true`, which is the "rule that cannot fire" this repo
# argues against everywhere else. gitleaks is worth having for the job it is
# actually good at (see truth/tasks/scan-the-tree-for-secrets-with-gitleaks.typ).
#
# Static, like check-doc-recipes.sh and script-scope-check.sh: it reads one
# committed JSON file. It doesn't run a recipe body, doesn't mint anything,
# and doesn't need lcov. This is the only CRAP-adjacent arm that is always
# correct rather than skipping when coverage is stale.
set -eo pipefail

baseline=.config/crap-baseline.json

if [ ! -f "$baseline" ]; then
    echo "crap-baseline-check: $baseline is missing — \`just crap::baseline\` mints it" >&2
    exit 1
fi

# An entries-less baseline passes every per-entry test below while disabling
# the ratchet: `--fail-regression` against an empty baseline reports
# every function as New and exits 0. Absent and empty are the same failure to
# the gate this file feeds, and this check refuses both.
count=$(jq '.entries | length' "$baseline")
if [ "$count" -eq 0 ]; then
    echo "crap-baseline-check: $baseline scores no functions — the ratchet would pass anything" >&2
    exit 1
fi

# A leading `/`, a `\\` UNC prefix or a `C:` drive letter is machine-local, and
# so is any path that climbs out of the repo with `..`. The filter checks
# Windows spellings even though the repo mints the file on Linux, because the
# failure they describe is a baseline minted on a Windows checkout. That is
# the one place a forward-slash test would silently pass a fully absolute path.
filter='
  .entries[]
  | .file
  | select(
      startswith("/")
      or startswith("\\")
      or test("^[A-Za-z]:")
      or (split("/") | any(. == ".."))
    )
'

# Positive control, because this gate's failure mode is a false green. A typo in the filter
# above doesn't match anything, and a filter that doesn't match anything is exactly what a
# clean baseline looks like. The check would print its success line over a file full of local
# paths. The same filter runs first over a fixture carrying one of each spelling, and the gate
# refuses to report on the real file unless the fixture trips it. The check uses `$filter`
# verbatim in both places rather than restating it, or the control would pin a duplicate the
# gate never runs.
fixture='{"entries":[
  {"file":"/home/u/repo/src/a.rs"},
  {"file":"C:/repo/src/b.rs"},
  {"file":"\\\\host\\share\\src\\c.rs"},
  {"file":"../outside/src/d.rs"},
  {"file":"crates/mindtape/src/ok.rs"}
]}'
control=$(jq -r "$filter" <<<"$fixture" | wc -l)
if [ "$control" -ne 4 ]; then
    echo "crap-baseline-check: the path filter flagged $control of 4 known-bad fixture paths" >&2
    echo "  it cannot be trusted to report on the real baseline — fix the filter, not the fixture" >&2
    exit 1
fi

offenders=$(jq -r "$filter" "$baseline" | sort -u)

if [ -n "$offenders" ]; then
    n=$(wc -l <<<"$offenders")
    echo "crap-baseline-check: $n of $count entries in $baseline carry a machine-local path" >&2
    head -5 <<<"$offenders" | while IFS= read -r path; do
        echo "  $path" >&2
    done
    [ "$n" -gt 5 ] && echo "  … and $((n - 5)) more" >&2
    echo "  re-mint with \`just crap::baseline\`, which strips the workspace root" >&2
    exit 1
fi
