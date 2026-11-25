#!/usr/bin/env bash
# .just/scripts/check-host-details.sh: the body of the `lint::host-details`
# recipe. Runs from the repo root, like the recipe that invokes it.
#
# Why the gate exists. A doc, a comment or a task that spells out a host
# setting reads as a requirement on the reader's machine. "This needs
# `kernel.perf_event_paranoid` at 1" tells a reader to reconfigure a kernel to
# run a benchmark, and the repo then carries one person's setup as if it were
# the project's. The same sentence in five files is five places to keep true.
#
# State the tool's property, never the host's setting. "callgrind and strace
# run unprivileged" is a fact about the tools and holds everywhere.
# "`kernel.perf_event_paranoid` must be 1" is a fact about one kernel.
#
# What it reads: one `git ls-files -z` pass, then one `awk` over the files it
# lists. No per-file process, because this arm runs in `_check-fast` beside
# every other fast gate.
#
# The tokens are the host settings a Linux machine carries, not every word
# that could describe one. A CPU model or a memory size reads like ordinary
# prose, so no scan catches those. This one catches the settings a reader
# would have to change.
#
# The escape hatch: `ignore`, a list of paths from the repo root. A doc that
# must spell out a setting to forbid it takes a path here. Both entries today
# are the rule's own text.
#
# The scan skips a tracked symlink. Git stores the link text, not the file, so
# the tracked content cannot hold a setting. Reading through the link would
# scan a target outside the tree, or the same file twice.
#
# The control lives in `_host-details-control.sh`, and the recipe runs it
# before this script. It drives this script over synthetic repos, so a control
# call here would call itself. check-secrets.sh gives the reason a control
# exists at all: "no findings" is equally what a clean tree prints and what a
# scanner that never opens a file prints.
set -eo pipefail

# Paths that may spell out a host setting, from the repo root. The scanner
# lists every token it refuses, and the control puts each one in a fixture, so
# both are the rule's own text.
ignore=(
    .just/scripts/check-host-details.sh
    .just/scripts/_host-details-control.sh
)

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

# shellcheck disable=SC2016 # the awk program is single-quoted on purpose
findings=$(tracked | xargs -0 -r awk '
    match($0, /sysctl|perf_event_paranoid|\/proc\/sys\/|setcap|CAP_SYS_[A-Z]+/) {
        printf "error: %s:%d names a host setting `%s`\n", FILENAME, FNR, substr($0, RSTART, RLENGTH)
    }
')

if [ -n "$findings" ]; then
    printf '%s\n' "$findings" >&2
    echo "check-host-details: a tracked file names a setting on the reader's machine" >&2
    echo "  state the tool's property instead, such as \"both run unprivileged\"" >&2
    echo "  a doc that must name one goes in the \`ignore\` array in .just/scripts/check-host-details.sh" >&2
    exit 1
fi
