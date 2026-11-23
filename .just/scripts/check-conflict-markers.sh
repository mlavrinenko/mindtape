#!/usr/bin/env bash
# .just/scripts/check-conflict-markers.sh: the body of the
# `lint::conflict-markers` recipe
# (truth/tasks/2026-11/add-a-gate-for-a-conflict-marker-left-in-a-tracked-file.typ).
# Runs from the repo root, like the recipe that invokes it.
#
# Why the gate exists. A hand-resolved merge can leave git's three marker
# lines in a tracked file, and no other gate reads a file for them. A
# compiler, a formatter and a prose linter all pass over the shape. Commit
# 28cafc4b landed a CHANGELOG.md holding all three, and 44 gates passed on
# that tree and on the next commit. Only a scan for the shape itself refuses
# it.
#
# What it reads: one `git ls-files -z` pass, then one `awk` over the files it
# names. No per-file process, because this arm runs in `_check-fast` beside
# every other fast gate. A line fails when it starts with `<<<<<<< `, starts
# with `>>>>>>> `, or is exactly `=======`.
#
# Why three lines and not "all three in one file". A half-resolved conflict
# keeps one or two of them, and that is the state a reader commits most
# often. `<<<<<<< ` and `>>>>>>> ` carry a trailing space and the branch
# name, so neither can be anything but a marker. `=======` alone is
# ambiguous: it is also a Markdown heading underline and a common prose
# separator. Git writes it as exactly seven `=` and nothing else, so the
# anchor is the whole line. The false-positive surface is a heading underline
# of exactly seven `=`, and no tracked file holds one today (checked with
# `git ls-files -z | xargs -0 awk '/^<<<<<<< / || /^=======$/ || /^>>>>>>> /'`).
# A doc that must quote a marker takes the escape hatch below rather than a
# looser rule here.
#
# The escape hatch: `ignore`, a list of paths from the repo root. A doc may
# quote a marker to document the shape, and a fenced example is prose to this
# scan. The array is empty today. Add a path only for a file that must show a
# marker, never to silence a real one.
#
# The scan skips a tracked symlink. Git stores the link text, not the file,
# so the tracked content cannot hold a marker. Reading through the link would
# scan a target outside the tree, or the same file twice (`CLAUDE.md` ->
# `AGENTS.md`).
#
# The control lives in `_conflict-markers-control.sh`, and the recipe runs it
# before this script. It drives this script over synthetic repos, so a
# control call here would call itself. check-secrets.sh gives the reason a
# control exists at all. "no leaks found" is equally what a clean tree prints
# and what a scanner that never opens a file prints.
set -eo pipefail

# Paths that may quote a marker, from the repo root. Empty today.
ignore=()

declare -A ignored=()
for path in "${ignore[@]}"; do
    ignored["$path"]=1
done

# One `git ls-files -z` pass, filtered here so the scan stays one `awk`
# process. `-z` keeps a path with a space in it whole.
tracked() {
    local path
    while IFS= read -r -d '' path; do
        [ -z "${ignored[$path]:-}" ] || continue
        [ ! -L "$path" ] || continue
        [ -f "$path" ] || continue
        printf '%s\0' "$path"
    done < <(git ls-files -z)
}

findings=$(tracked | xargs -0 -r awk '
    /^<<<<<<< / || /^=======$/ || /^>>>>>>> / {
        printf "error: %s:%d has a conflict marker `%s`\n", FILENAME, FNR, substr($0, 1, 7)
    }
')

if [ -n "$findings" ]; then
    printf '%s\n' "$findings" >&2
    echo "check-conflict-markers: a tracked file holds a git conflict marker" >&2
    echo "  resolve the conflict and remove the marker" >&2
    echo "  a doc that must quote one goes in the \`ignore\` array in .just/scripts/check-conflict-markers.sh" >&2
    exit 1
fi
