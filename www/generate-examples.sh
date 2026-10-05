#!/usr/bin/env bash
# www/generate-examples.sh — slice the checked example projects into facts the
# docs pages quote verbatim, the way www/generate-facts.sh slices the README
# task. Called by www/generate.sh with the repo root and the generated/
# directory, inside the lock that script already holds, so this one takes none
# of its own.
#
# One artifact per fixture: examples/links/ -> links.json (the `links:` field),
# examples/recurring/ -> recurring.json (the `recur:` field),
# examples/statuses/ -> statuses.json (the `status:` field), examples/dsl/ ->
# dsl.json (the task sketch AGENTS.md and /dsl/ show), examples/self-validating/ ->
# self-validating.json (a project's own check function), examples/priority/ ->
# priority.json (a framework call), examples/api-service/ -> api-service.json
# (the task README.md opens with). The pages that quote
# them read only the slices, and `mt check` runs on each fixture, so a helper
# whose shape changes fails that check before a page can show it.
#
# The marker convention, so a fixture stays a readable project and not a doc
# fragment: any .typ file in the fixture marks the lines worth quoting with
#
#   // docs:slice <id>
#   ...code...
#   // docs:end
#
# `<id>` is the JSON key the page reads. Markers may repeat across files, the
# code keeps its own indentation, and an unterminated slice is a hard error
# rather than a silently empty block. A slice id a page declares but the
# fixture does not carry fails that page's render (`<id>` missing from the
# fact); a slice no page quotes is invisible, so pages assert the whole key set
# instead of only probing the ones they read.
#
# The second half slices config rather than task markers — a `toml` key or a
# whole section out of a project's `.mindtape/config.toml`, under the same
# discipline in TOML's comment syntax:
#
#   # docs:slice <fact>/<id>
#   ...toml...
#   # docs:end
#
# The fact name rides in the marker here because one config file feeds several
# pages — the repo's own carries /hooks/'s run-less gate and /rule-packs/'s
# adopt block — where a task-slice fixture is one project per page and can take its
# name from the directory. Each fact lands at <fact>-config.json, shaped like a
# task fact minus `root`: one fact's slices may come from two projects, so the
# `file` field is repo-relative on its own. Suffix, not prefix: www/generate.sh
# already writes `config-schema.json` (the `mt config schema` capture), and a
# fact named for the /schema/ page would have overwritten it silently.
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
  local root="$REPO/examples/$name"
  local dir="$tmp/$name"
  [ -d "$root" ] || {
    echo "generate-examples.sh: examples/$name/ is missing" >&2
    return 1
  }
  mkdir -p "$dir"

  # Every .typ file in the fixture, so a slice can come from a task or from a
  # module the tasks import. The counter keeps two files with one stem apart.
  local f n=0
  while IFS= read -r f; do
    n=$((n + 1))
    awk -v file="${f#"$root"/}" -v dir="$dir" -v n="$n" '
      /^[[:space:]]*\/\/ docs:slice / {
        id = $NF
        out = dir "/" n "." id ".code"
        capture = 1
        next
      }
      /^[[:space:]]*\/\/ docs:end/ {
        if (capture) { close(out); print file "\t" id "\t" out }
        capture = 0
        next
      }
      capture { print > out }
      END { if (capture) { print "generate-examples.sh: unterminated docs:slice in " FILENAME > "/dev/stderr"; exit 1 } }
    ' "$f"
  done < <(find "$root" -name '*.typ' | sort) >"$dir/index.tsv"

  [ -s "$dir/index.tsv" ] || {
    echo "generate-examples.sh: no docs:slice blocks under examples/$name" >&2
    return 1
  }

  {
    while IFS=$'\t' read -r file id path; do
      jq -n \
        --arg id "$id" \
        --arg file "$file" \
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
slice_fixture dsl | emit dsl.json
slice_fixture statuses | emit statuses.json
slice_fixture self-validating | emit self-validating.json
slice_fixture priority | emit priority.json
slice_fixture api-service | emit api-service.json

# Every config file scanned for config markers, repo-relative. A project not
# listed here is simply not read, so a marker added to one is inert until its
# path arrives — the same explicitness the `examples` mmz scope holds the
# checked fixtures to, for the same reason.
CONFIG_SOURCES=(
  ".mindtape/config.toml"
  "examples/api-service/.mindtape/config.toml"
  "examples/named-sets/.mindtape/config.toml"
  "examples/priority/.mindtape/config.toml"
  "examples/estimates/.mindtape/config.toml"
  "examples/links/.mindtape/config.toml"
  "examples/rules/.mindtape/config.toml"
  "examples/statuses/.mindtape/config.toml"
  "examples/derived-tags/.mindtape/config.toml"
  "examples/self-validating/.mindtape/config.toml"
)

# Every fact a page opens. Declared rather than derived from the markers
# because a mistyped fact name would otherwise emit a file no page ever reads —
# invisible, and the one drift the per-page key-set assertion in
# www/utils/slices.typ structurally cannot see from inside a single page.
# `api-service` holds the problem slices. The Problems pages render them
# through www/utils/problems.typ, and generate-api-service.sh reads the same
# markers to require a session for each.
CONFIG_FACTS=(api-service commit-rules config derived-tags estimates hooks links named-sets priority rule-packs rules self-validating statuses)

# Same two-pass shape as slice_fixture: awk writes each slice to its own temp
# file and one index row per slice, then jq builds the JSON with --rawfile so
# no escaping rule lives in awk.
slice_configs() {
  local dir="$tmp/config"
  mkdir -p "$dir"

  local src
  for src in "${CONFIG_SOURCES[@]}"; do
    [ -f "$REPO/$src" ] || {
      echo "generate-examples.sh: $src is missing" >&2
      return 1
    }
    awk -v src="$src" -v dir="$dir" '
      /^[[:space:]]*# docs:slice / {
        if (split($NF, parts, "/") != 2) {
          print "generate-examples.sh: docs:slice marker in " FILENAME " reads " $NF ", want <fact>/<id>" > "/dev/stderr"
          bad = 1
          exit 1
        }
        fact = parts[1]
        id = parts[2]
        out = dir "/" fact "." id ".code"
        capture = 1
        next
      }
      /^[[:space:]]*# docs:end/ {
        if (capture) { close(out); print fact "\t" id "\t" src "\t" out }
        capture = 0
        next
      }
      capture { print > out }
      END { if (capture && !bad) { print "generate-examples.sh: unterminated docs:slice in " FILENAME > "/dev/stderr"; exit 1 } }
    ' "$REPO/$src"
  done >"$dir/index.tsv"

  local found declared
  found="$(cut -f1 "$dir/index.tsv" | sort -u | tr '\n' ' ')"
  declared="$(printf '%s\n' "${CONFIG_FACTS[@]}" | sort -u | tr '\n' ' ')"
  [ "$found" = "$declared" ] || {
    echo "generate-examples.sh: config markers name facts ($found); this script declares ($declared)" >&2
    return 1
  }

  local fact
  for fact in "${CONFIG_FACTS[@]}"; do
    {
      while IFS=$'\t' read -r f id file path; do
        [ "$f" = "$fact" ] || continue
        jq -n \
          --arg id "$id" \
          --arg file "$file" \
          --rawfile code "$path" \
          '{id: $id, file: $file, code: ($code | rtrimstr("\n"))}'
      done <"$dir/index.tsv"
    } | jq -s --arg fact "$fact" '
        reduce .[] as $s (
          {slices: {}};
          ($s.id) as $i
          | if (.slices | has($i)) then
              error("generate-examples.sh: duplicate slice id \($i) in config fact \($fact)")
            else
              .slices[$i] = {file: $s.file, code: $s.code}
            end
        )
      ' | emit "$fact-config.json"
  done
}

slice_configs
