#!/usr/bin/env bash
# .just/scripts/wt-carry-lcov.sh <primary-checkout> — the worktrunk `pre-remove`
# hook's lcov carry (.config/wt.toml). Runs from the root of the worktree about
# to be removed, which is where worktrunk runs that hook.
#
# `just cover` writes target/coverage/lcov.info, and `just crap::check` scores against
# it. The file is gitignored, so after a fast-forward the primary checkout holds
# none, or one from before the merge, and the next `just crap::check` there would pay
# a second instrumented build for numbers the worktree already has.
#
# Never fatal: the merge has landed, and a failed copy does not un-land it. Every
# way out exits 0, and a skip for staleness or a failed copy says so on stderr.
set -eo pipefail

primary=${1:?usage: wt-carry-lcov.sh <primary-checkout>}
lcov=target/coverage/lcov.info
here=$(git rev-parse --show-toplevel) || exit 0

[ "$here" != "$primary" ] || exit 0
# Only a landed merge carries: otherwise the lcov describes a tree the primary
# checkout does not hold. A primary that is no checkout answers nothing, which
# reads as not landed.
[ "$(git rev-parse HEAD)" = "$(git -C "$primary" rev-parse HEAD 2>/dev/null)" ] || exit 0
[ -f "$here/$lcov" ] || exit 0
# The primary's own lcov wins when it is newer.
if [ -f "$primary/$lcov" ] && [ ! "$here/$lcov" -nt "$primary/$lcov" ]; then
    exit 0
fi

# The copy gets a new mtime, so staleness is judged here, against the sources
# the lcov was written from, by the same script `just crap::check` asks. That script
# judges by content through mmz, so a rebase that rewrites mtimes but not bytes
# still carries.
stale=$(cd "$here" && bash .just/scripts/_lcov-stale-reason.sh 2>/dev/null) || stale=unanswered
if [ -n "$stale" ]; then
    echo "wt-carry-lcov: skipped, $stale" >&2
    exit 0
fi

# llvm-cov writes absolute `SF:` paths, so they are rewritten to the primary
# checkout. The paths go in through the environment: awk -v would read a
# backslash in one as an escape.
if mkdir -p "$primary/target/coverage" \
    && FROM="SF:$here/" TO="SF:$primary/" awk '
        BEGIN { from = ENVIRON["FROM"]; to = ENVIRON["TO"] }
        index($0, from) == 1 { $0 = to substr($0, length(from) + 1) }
        { print }
    ' "$here/$lcov" >"$primary/$lcov.tmp" \
    && mv "$primary/$lcov.tmp" "$primary/$lcov"; then
    exit 0
fi
echo "wt-carry-lcov: could not carry $lcov to $primary" >&2
exit 0
