#!/usr/bin/env bash
# .just/scripts/check-commit-prose.sh [MESSAGE_FILE]: the commit message's
# prose, through Vale. `mt check --commit-msg` and `mt check --commits` run it
# through the `[commits] run` command in .mindtape/config.toml, with the
# message on stdin. It is the project's policy, not `mt`'s: mindtape only runs
# the command and reports each line it prints. Runs from the repo root.
#
# This script reads the message from the file argument, or from stdin when there is
# none. Comment lines and a `git commit -v` diff after the scissors line never reach a
# commit, and the script drops them before Vale reads the text. The script skips a
# `fixup!`, `squash!` or `amend!` message for the same reason: autosquash folds it
# into a commit that the style checks.
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
