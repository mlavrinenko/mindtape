#!/usr/bin/env bash
# www/generate-task-files.sh — the captures /task-files/ shows. www/generate.sh
# calls it inside the lock it holds, so this script takes no lock of its own.
#
# The page walks examples/task-files/tasks/fix-the-login-redirect.typ, so this
# copies it under generated/ without its docs:slice markers, as generate.sh
# does for the hero source. Then it copies each broken file from
# examples/task-files/invalid/ into a scratch copy of the project, captures the
# `mt check` that refuses it, and removes it again, so each record shows one
# error. Each capture expects exit 1 and the error the page explains, so the
# build fails if a mistake starts passing or fails for another reason.
set -eo pipefail

REPO="$1"
OUT="$2"
MT="$REPO/target/debug/mt"
FIX=examples/task-files

grep -v '^[[:space:]]*// docs:' "$REPO/$FIX/tasks/fix-the-login-redirect.typ" >"$OUT/task-files-source.typ"

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
capture_init
capture_copy TF "$FIX"

# check_invalid <stem> <regex>: capture `mt check` with invalid/<stem>.typ
# among the tasks, as the record task-files-<stem>.
check_invalid() {
  cp "$TF/invalid/$1.typ" "$TF/tasks/$1.typ"
  capture --exit 1 --expect "$2" "task-files-$1" "$TF" 'mt check'
  rm "$TF/tasks/$1.typ"
}

check_invalid missing-comma 'expected comma'
check_invalid dictionary-in-braces 'unexpected colon'
check_invalid one-tag-without-a-comma 'tags: expected an array, found a string'
check_invalid note-as-a-string 'note must be a content block'
check_invalid links-joined-with-a-comma 'no positional args besides its body'
check_invalid hash-in-the-body 'unknown variable: backend'
