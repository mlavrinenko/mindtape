#!/usr/bin/env bash
# www/generate-examples.sh — slice the checked example projects into facts the
# docs pages quote verbatim, the way www/generate-facts.sh slices the README
# task. Called by www/generate.sh with the repo root and the generated/
# directory, inside the lock that script already holds, so this one takes none
# of its own.
#
# One artifact per fixture: examples/links/ -> links.json (the `links:` field),
# examples/recurring/ -> recurring.json (the `recur:` field). The pages
# www/content/links.typ and www/content/recurring.typ read only the slices, and
# the fixtures are what `just check-examples` validates; a helper whose shape
# changes breaks the fixture's check before a page can lie about it.
#
# The marker convention, so a fixture stays a readable project and not a doc
# fragment: a task file marks the lines worth quoting with
#
#   // docs:slice <id>
#   ...code...
#   // docs:end
#
# `<id>` is the JSON key the page reads. Markers may repeat across files, the
# code keeps its own indentation, and an unterminated slice is a hard error
# rather than a silently empty block. A slice id a page declares but the
# fixture no longer carries fails that page's render (`<id>` missing from the
# fact); a slice no page quotes is invisible, so pages assert the whole key set
# instead of only probing the ones they read.
set -eo pipefail

REPO="$1"
OUT="$2"
if [ -z "$REPO" ] || [ -z "$OUT" ]; then
  echo "usage: generate-examples.sh <repo-root> <generated-dir>" >&2
  exit 1
fi

# Same temp-file-plus-atomic-mv discipline as generate-facts.sh's emit(): a
# reader mid-render sees the previous complete file or the next, never a
# truncated one, and a producer that dies leaves the old artifact in place.
emit() {
  local name="$1" tmp="$OUT/.$1.tmp.$$"
  cat >"$tmp"
  if ! jq -e . "$tmp" >/dev/null 2>&1; then
    rm -f "$tmp"
    echo "generate-examples.sh: refusing to install malformed $name" >&2
    return 1
  fi
  mv -f "$tmp" "$OUT/$name"
}

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# One record per `docs:slice` block: which file it came from, and the code as
# written (indentation kept — the page shows it inside `task.with(...)`, so
# dedenting would be the first lie). The awk pass writes each slice to its own
# temp file so jq reads the code with `--rawfile`; building JSON inside awk
# would put the escaping rules in the wrong program.
slice_fixture() {
  local name="$1"
  local src="$REPO/examples/$name/tasks"
  local dir="$tmp/$name"
  [ -d "$src" ] || {
    echo "generate-examples.sh: examples/$name/tasks/ is missing" >&2
    return 1
  }
  mkdir -p "$dir"

  local f stem
  for f in "$src"/*.typ; do
    stem="$(basename "$f" .typ)"
    awk -v stem="$stem" -v dir="$dir" '
      /^[[:space:]]*\/\/ docs:slice / {
        id = $NF
        out = dir "/" stem "." id ".code"
        capture = 1
        next
      }
      /^[[:space:]]*\/\/ docs:end/ {
        if (capture) { close(out); print stem "\t" id "\t" out }
        capture = 0
        next
      }
      capture { print > out }
      END { if (capture) { print "generate-examples.sh: unterminated docs:slice in " FILENAME > "/dev/stderr"; exit 1 } }
    ' "$f"
  done >"$dir/index.tsv"

  [ -s "$dir/index.tsv" ] || {
    echo "generate-examples.sh: no docs:slice blocks under examples/$name/tasks" >&2
    return 1
  }

  {
    while IFS=$'\t' read -r stem id path; do
      jq -n \
        --arg id "$id" \
        --arg file "tasks/$stem.typ" \
        --rawfile code "$path" \
        '{id: $id, file: $file, code: ($code | rtrimstr("\n"))}'
    done <"$dir/index.tsv"
  } | jq -s --arg root "examples/$name" '
      reduce .[] as $s (
        {root: $root, slices: {}};
        ($s.id) as $i
        | if (.slices | has($i)) then
            error("generate-examples.sh: duplicate slice id \($i) under \($root)")
          else
            .slices[$i] = {file: $s.file, code: $s.code}
          end
      )
    '
}

slice_fixture links | emit links.json
slice_fixture recurring | emit recurring.json
