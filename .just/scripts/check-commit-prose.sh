#!/usr/bin/env bash
# .just/scripts/check-commit-prose.sh [MESSAGE_FILE] — the commit message's
# prose, through Vale. `mt check --commit-msg` and `mt check --commits` run it
# through the `[commits] run` command in .mindtape/config.toml, with the
# message on stdin. It is the project's policy, not `mt`'s: mindtape only runs
# the command and reports each line it prints. Runs from the repo root.
#
# The message is read from the file argument, or stdin when there is none.
# Comment lines and a `git commit -v` diff after the scissors line never reach
# a commit, so they are dropped before Vale reads the text. A `fixup!`,
# `squash!` or `amend!` message is skipped for the same reason: autosquash
# folds it into a commit that the style checks.
set -eo pipefail

raw=$(mktemp)
msg=$(mktemp)
trap 'rm -f "$raw" "$msg"' EXIT

if [ "$#" -gt 0 ]; then
    cat "$1" >"$raw"
else
    cat >"$raw"
fi

awk '
    /^# -* >8 -*$/ { exit }
    /^#/ { next }
    { print }
' "$raw" >"$msg"

case "$(sed -n '1p' "$msg")" in
    'fixup! '* | 'squash! '* | 'amend! '*) exit 0 ;;
esac

vale --no-global --config=.config/vale-commits.ini --output=line <"$msg"
