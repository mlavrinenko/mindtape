#!/usr/bin/env bash
# ask-refactoring.sh — turn advisory quality signals into backlog tasks.
#
# Run it as `just ask-refactoring` (dry) or `just ask-refactoring --write`.
#
# Mechanical refactoring work recurs, and a human notices it only when it
# blocks something else. The tools that predict it are machine-readable; this
# reads them into the backlog. Three rules the design turns on:
#
#   Only advisory signals qualify. A signal that already blocks a commit, such
#   as jscpd's new clones, outdatty, machete or clippy, never files a task. The
#   part of jscpd no gate enforces is the baseline `just lint::dry` tolerates,
#   which the dry arm files, and the snippets under its 50-token floor, which
#   the repeats arm files.
#
#   One task per finding, never one per sweep: each goes to its own agent.
#
#   Idempotency comes from the signal, not a ledger. A closed task whose
#   condition still holds is noticed again, because the work did not happen.
#   The one suppression is an override with a written reason in the signal's
#   own config: .config/linecop.yaml for sizes, an `--exclude` in the
#   `just crap::check` recipe for coverage.
#
# The re-run guard is [views.ask-refactoring] in .mindtape/config.toml: while a
# task the sweep filed is open, the sweep refuses.
#
# Each signal is one file under `contrib/ask-refactoring/`, named in `ARMS`.
# `signal_<arm>` prints TSV rows on stdout and `emit_<arm>` turns a row into a
# task, so a signal that shells out to a producer sends that output to stderr
# (see `ensure_lcov_fresh`). The tags and the guard need no change for a new arm.
#
# Not memoized with mmz: the point is to re-observe the tree every run. Only
# `just mmz cover` inside the undertested arm is cached, and the lcov it reads
# is still checked for freshness every sweep.
set -eo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"

# ── the thresholds ──────────────────────────────────────────────────────────
#
# All six are required and none is defaulted here. The defaults live only in
# the recipe's `[arg(...)]` attributes, which `just --usage`, the generated
# sweep-flags.json and docs/contributing/signal-sweep.md all read. A fallback
# here would be a second copy that fails silently at a threshold nobody chose.
#
# Initialised empty because the recipe exports the same names, so an inherited
# value would otherwise pass the required-check without the flag.
BASELINE=""
CC_FLOOR=""
FN_COVERAGE_FLOOR=""
REPEAT_GAIN=""
DRY_GAIN=""
WRITE=""

usage() {
    cat <<'EOF'
usage: bash contrib/ask-refactoring.sh --write 0|1 --baseline PCT
           --cc-floor N --fn-coverage-floor PCT --repeat-gain N --dry-gain N

Run it as `just ask-refactoring` — the recipe owns every default and forwards
all six flags; `just --usage ask-refactoring` prints them with their values.
All six are required here; there are no defaults at this layer.

env:
  MT   path to the `mt` binary (default: build and use ./target/debug/mt)
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --write) WRITE="$2"; shift 2 ;;
        --write=*) WRITE="${1#*=}"; shift ;;
        --baseline) BASELINE="$2"; shift 2 ;;
        --baseline=*) BASELINE="${1#*=}"; shift ;;
        --cc-floor) CC_FLOOR="$2"; shift 2 ;;
        --cc-floor=*) CC_FLOOR="${1#*=}"; shift ;;
        --fn-coverage-floor) FN_COVERAGE_FLOOR="$2"; shift 2 ;;
        --fn-coverage-floor=*) FN_COVERAGE_FLOOR="${1#*=}"; shift ;;
        --repeat-gain) REPEAT_GAIN="$2"; shift 2 ;;
        --repeat-gain=*) REPEAT_GAIN="${1#*=}"; shift ;;
        --dry-gain) DRY_GAIN="$2"; shift 2 ;;
        --dry-gain=*) DRY_GAIN="${1#*=}"; shift ;;
        -h | --help) usage; exit 0 ;;
        *) echo "ask-refactoring: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

# Refuse a missing or non-numeric flag. The recipe's patterns catch these at
# dispatch, so this guards a direct `bash contrib/ask-refactoring.sh` call.
for var in WRITE BASELINE CC_FLOOR FN_COVERAGE_FLOOR REPEAT_GAIN DRY_GAIN; do
    flag="--$(printf '%s' "$var" | tr 'A-Z_' 'a-z-')"
    value="${!var}"
    if [ -z "$value" ]; then
        echo "ask-refactoring: missing $flag (no default at this layer)" >&2
        usage >&2
        exit 2
    fi
    case "$value" in
        *[!0-9]*)
            echo "ask-refactoring: $flag wants a number, got '$value'" >&2
            exit 2
            ;;
    esac
done

# Use the worktree's own `mt`, never the one on PATH, which may predate the
# `--dedup` and `--format json` the sweep needs.
MT="${MT:-}"
if [ -z "$MT" ]; then
    cargo build -p mindtape-cli -q
    MT="$repo/target/debug/mt"
fi

# ── the re-run guard ────────────────────────────────────────────────────────
open="$("$MT" ls '@ask-refactoring' --format path)"
if [ -n "$open" ]; then
    count="$(printf '%s\n' "$open" | grep -c .)"
    echo "ask-refactoring: $count task(s) from the last sweep are still open:" >&2
    printf '%s\n' "$open" | sed 's/^/  /' >&2
    cat >&2 <<'EOF'

Empty the view before sweeping again — finish them, cancel them, or write the
override that makes the finding stop existing. Filing a second sprint on top of
the first is how a signal-driven backlog becomes noise.

  mt ls @ask-refactoring
EOF
    exit 1
fi

# A stem from the finding, not the title: stable across runs, so `--dedup
# skip` knows a finding again, and prefixed by the arm (`$1`) so arms cannot
# collide. `$2` is the finding. Operators are spelled out first, or
# `replace > with >=` and `replace > with <` fold to one stem and the second
# is dropped as already filed.
stem_for() {
    local encoded
    encoded="$(printf '%s' "$2" \
        | sed -e 's/&&/and/g' -e 's/||/or/g' \
              -e 's/!=/ne/g' -e 's/==/eq/g' -e 's/>=/ge/g' -e 's/<=/le/g' \
              -e 's/->/to/g' -e 's/+=/plus-eq/g' -e 's/-=/minus-eq/g' -e 's/\*=/star-eq/g' \
              -e 's/>/gt/g' -e 's/</lt/g' -e 's/!/not/g' -e 's/+/plus/g' \
              -e 's/\*/star/g' -e 's/%/percent/g' -e 's/\^/caret/g' -e 's/~/tilde/g' \
              -e 's/=/eq/g' -e 's/&/and/g' -e 's/|/or/g')"
    printf '%s-%s\n' "$1" "$(printf '%s' "$encoded" | tr -c 'A-Za-z0-9' '-' | tr -s '-' | sed 's/^-//; s/-$//')"
}

# File one finding, or print what would be filed. `$1` is the arm, `$2` the
# stem, `$3` a one-line summary for the dry run, `$4` the arm's advice for a
# skipped finding (one line per newline: why the stem is taken and which
# override clears it), and the rest the `mt` args (`add` and on). Shared, so an
# arm's emit only builds its args and words its own advice. Every arm uses it.
file_finding() {
    local arm="$1" stem="$2" summary="$3" advice="$4"
    shift 4
    # Every batch lands in one folder, so `mt ls swept` lists it.
    set -- "$@" --to swept
    if [ "$WRITE" -eq 1 ]; then
        local out outcome dest
        out="$("$MT" "$@" --dedup skip --format json)"
        outcome="$(printf '%s' "$out" | jq -r '.outcome')"
        dest="$(printf '%s' "$out" | jq -r '.path')"
        case "$outcome" in
            wrote) echo "filed   $dest" ;;
            skipped)
                echo "skipped $dest" >&2
                printf '%s\n' "$advice" | sed 's/^/        /' >&2
                ;;
            *) echo "$outcome $dest" ;;
        esac
    else
        # Dry: queue the finding under its ICE score; `report_arm` renders
        # only the top ones, since `mt add --print` costs a second apiece.
        local score
        score="$(printf '%s\n' "$@" | awk -F= '
            /^(impact|confidence|ease)=/ { v[$1] = $2 }
            END { printf "%.2f", v["impact"] * v["confidence"] * v["ease"] }')"
        queued=$((queued + 1))
        printf '%s\0' "$@" >"$queue/$queued.args"
        printf '%s\t%s\t%s\t%s\n' "$score" "$queued" "$stem" "$summary" >>"$queue/$arm.tsv"
    fi
}

# Dry run: print an arm's count and its top findings by ICE score, and write
# those findings' bodies under `$preview`, where they are read before
# `--write` files them. `$1` is the arm, `$2` its finding count.
report_arm() {
    local arm="$1" count="$2" score n stem summary
    if [ -e "$queue/$arm.skipped" ]; then
        printf '%-12s skipped, see above\n' "$arm"
        return 0
    fi
    printf '%-12s %4d finding(s)\n' "$arm" "$count"
    [ "$count" -gt 0 ] || return 0
    while IFS=$'\t' read -r score n stem summary; do
        local args=()
        mapfile -d '' -t args <"$queue/$n.args"
        "$MT" "${args[@]}" --print >"$preview/$stem.typ"
        printf '  %s  %s\n' "$stem" "$summary"
    done < <(sort -t$'\t' -k1,1nr -k3,3 "$queue/$arm.tsv" | head -n "$TOP")
}

# An arm that cannot measure calls this after saying why on stderr, so the
# dry run reports it skipped rather than clean.
skip_arm() {
    : >"$queue/$1.skipped"
}

# ── the arms ────────────────────────────────────────────────────────────────
#
# Each file defines `score_<arm>`, `signal_<arm>` and `emit_<arm>`, with the
# reasoning behind its numbers. To add an arm, add a file, its name in `ARMS`
# and a template.
ARMS=(size undertested mutants repeats dry)

for arm in "${ARMS[@]}"; do
    # shellcheck source=/dev/null
    . "$repo/contrib/ask-refactoring/$arm.sh"
done

# Each signal's rows are captured with `$(...)`, not read off `< <(...)`:
# only the command substitution carries a signal's `exit 1` through `set -e`.
# Through a process substitution a refusing signal would read as zero rows.
#
# How many findings a dry run shows per arm; it counts the rest.
TOP=3
queue="$(mktemp -d)"
trap 'rm -rf "$queue"' EXIT
queued=0
preview="$repo/target/ask-refactoring"
if [ "$WRITE" -eq 0 ]; then
    rm -rf "$preview"
    mkdir -p "$preview"
fi

findings=0
for arm in "${ARMS[@]}"; do
    rows="$("signal_$arm")"
    before="$findings"
    while IFS=$'\t' read -r -a row; do
        [ "${#row[@]}" -gt 0 ] || continue
        "emit_$arm" "${row[@]}"
        findings=$((findings + 1))
    done <<<"$rows"
    [ "$WRITE" -eq 1 ] || report_arm "$arm" "$((findings - before))"
done

if [ "$findings" -eq 0 ]; then
    echo "ask-refactoring: no findings at --baseline $BASELINE" \
        "/ --cc-floor $CC_FLOOR --fn-coverage-floor $FN_COVERAGE_FLOOR / --repeat-gain $REPEAT_GAIN / --dry-gain $DRY_GAIN"
elif [ "$WRITE" -eq 0 ]; then
    echo "ask-refactoring: $findings finding(s); the bodies shown are in ${preview#"$repo"/}/;" \
        "re-run with --write to file them all"
else
    echo "ask-refactoring: $findings finding(s) swept"
fi
