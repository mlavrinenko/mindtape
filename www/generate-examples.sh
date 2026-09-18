#!/usr/bin/env bash
# www/generate-examples.sh — slice the checked example projects into facts the
# docs pages quote verbatim, the way www/generate-facts.sh slices the README
# task. Called by www/generate.sh with the repo root and the generated/
# directory, inside the lock that script already holds, so this one takes none
# of its own.
#
# Why a slice and not a quote. A doc code block is a claim about the DSL, and
# the only way it stays true is if it IS the file: `just check-examples` runs
# `mt check` over examples/links/, and this script copies the marked lines out
# of the files it validated. A helper whose shape changes breaks the fixture's
# check before the page can lie about it.
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
# fixture no longer carries fails that page's render (`<id>` missing from
# links.json); a slice no page quotes is invisible, so pages assert the whole
# key set instead of only probing the ones they read.
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

LINKS="$REPO/examples/links/tasks"
[ -d "$LINKS" ] || {
  echo "generate-examples.sh: examples/links/tasks/ is missing" >&2
  exit 1
}

# --- links.json ---------------------------------------------------------------
# One record per `docs:slice` block: which file it came from, and the code as
# written (indentation kept — the page shows it inside `task.with(...)`, so
# dedenting would be the first lie). The awk pass writes each slice to its own
# temp file so jq reads the code with `--rawfile`; building JSON inside awk
# would put the escaping rules in the wrong program.
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

for f in "$LINKS"/*.typ; do
  stem="$(basename "$f" .typ)"
  awk -v stem="$stem" -v dir="$tmp" '
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
done >"$tmp/index.tsv"

[ -s "$tmp/index.tsv" ] || {
  echo "generate-examples.sh: no docs:slice blocks under examples/links/tasks" >&2
  exit 1
}

{
  while IFS=$'\t' read -r stem id path; do
    jq -n \
      --arg id "$id" \
      --arg file "tasks/$stem.typ" \
      --rawfile code "$path" \
      '{id: $id, file: $file, code: ($code | rtrimstr("\n"))}'
  done <"$tmp/index.tsv"
} | jq -s '
    reduce .[] as $s (
      {root: "examples/links", slices: {}};
      ($s.id) as $i
      | if (.slices | has($i)) then
          error("generate-examples.sh: duplicate slice id \($i)")
        else
          .slices[$i] = {file: $s.file, code: $s.code}
        end
    )
  ' | emit links.json
