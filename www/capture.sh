# www/capture.sh — record commands and what they printed, for the docs pages.
# A www/generate-*.sh script sources it, sets REPO, OUT and MT, and calls
# capture_init before its first capture.
#
# A record is www/generated/<slug>.json:
#
#   {"project": "examples/api-service",
#    "steps": [{"cmd": "mt ls", "out": "...", "code": 0}]}
#
# `cmd` is the line bash ran, `out` is what the terminal showed (stdout and
# stderr, ANSI colour included) and `code` is the exit status. A page passes
# the slug to terminal() or commands() in www/utils/ui.typ, which print each
# command from the record, so a page cannot show a command that did not run.
#
# Each command runs under a pseudo-terminal 100 columns wide, because the
# reader runs it in one and mt prints a different table off a terminal: a
# `file` column, and no status glyphs. `script` comes from util-linux in the
# dev shell. `mt` on PATH is the binary under test, and the environment
# variables that change what mt prints are pinned or cleared.
#
# Each absolute path to the project is cut to a project-relative one, and
# each CAPTURE_PATHS "from=to" pair is replaced. An output that still names
# the capture area fails the build.
#
# Calls with one slug form a session: the steps run in order in one project,
# and each sees what the steps before it wrote. A step that exits other than
# its expected code (0 unless --exit says otherwise) fails the build, so a
# command a page shows cannot start failing, and a refusal it shows cannot
# start passing.

CAPTURE_PATHS=()

# Make the capture area: the `mt` shim on PATH, and a data home holding this
# build's prelude for projects that resolve `@local/mindtape` from it. The
# config home stays empty, so a builder's own defaults.toml changes nothing.
capture_init() {
    CAPTURE_TMP=$(cd "$(mktemp -d)" && pwd -P)
    trap 'rm -rf "$CAPTURE_TMP"' EXIT
    mkdir -p "$CAPTURE_TMP/bin"
    ln -s "$MT" "$CAPTURE_TMP/bin/mt"
    CAPTURE_CONFIG_HOME=
    CAPTURE_DATA_HOME="$CAPTURE_TMP/data"
    MINDTAPE_DATA_HOME="$CAPTURE_DATA_HOME" "$MT" install \
        --no-defaults --no-schema --no-completions >/dev/null
    local found
    found=$(_capture_run "$CAPTURE_TMP" 'command -v mt')
    if [ "$found" != "$CAPTURE_TMP/bin/mt" ]; then
        echo "capture_init: mt in a capture resolves to $found, not the binary under test" >&2
        return 1
    fi
}

# capture_copy <var> <fixture>: copy a project under the repo, such as
# examples/links, for commands that write, and store the copy's path in <var>.
# The copy keeps the fixture's depth below a symlinked lib/, so a
# `[[typst.local-packages]]` path such as ../../lib reaches this checkout's
# prelude.
capture_copy() {
    local root
    root=$(mktemp -d "$CAPTURE_TMP/copy.XXXX")
    mkdir -p "$root/$(dirname "$2")"
    ln -s "$REPO/lib" "$root/lib"
    cp -r "$REPO/$2" "$root/$2"
    printf -v "$1" '%s' "$root/$2"
}

# capture_scratch <var>: an empty directory for a project the captures make.
capture_scratch() {
    local root
    root=$(mktemp -d "$CAPTURE_TMP/scratch.XXXX")
    mkdir "$root/scratch"
    printf -v "$1" '%s' "$root/scratch"
}

# capture_home: give the captures that follow an empty config and data home,
# as on a machine where mt was never installed. Their paths print as the
# Linux defaults. Call it in a subshell to scope it to one session.
capture_home() {
    local root
    root=$(mktemp -d "$CAPTURE_TMP/home.XXXX")
    mkdir "$root/config" "$root/data"
    CAPTURE_CONFIG_HOME="$root/config" CAPTURE_DATA_HOME="$root/data"
    CAPTURE_PATHS+=("$root/config=~/.config" "$root/data=~/.local/share")
}

# The project the record names: the directory's path in the repo, or in the
# area capture_copy or capture_scratch made it in.
_capture_project() {
    case "$1" in
    "$CAPTURE_TMP"/*/*) local rest=${1#"$CAPTURE_TMP"/} && echo "${rest#*/}" ;;
    "$REPO") echo . ;;
    "$REPO"/*) echo "${1#"$REPO"/}" ;;
    *) echo "capture: $1 is outside the repo and the capture area" >&2 && return 1 ;;
    esac
}

# Run <command> in <dir> the way every capture runs one, printing what the
# terminal showed and exiting with the command's status. BASH_ENV is cleared
# because a startup file it names can rewrite PATH, and then `mt` resolves to
# whatever binary the builder has installed.
_capture_run() { # <dir> <command>
    cd "$1" && env -u BASH_ENV -u NO_COLOR -u COLUMNS -u LINES -u EDITOR -u VISUAL \
        -u MINDTAPE_WORKTREE -u MINDTAPE_PRINCIPAL -u MINDTAPE_NO_CACHE \
        TERM=xterm-256color SHELL="$(command -v bash)" PATH="$CAPTURE_TMP/bin:$PATH" \
        MINDTAPE_CONFIG_HOME="$CAPTURE_CONFIG_HOME" MINDTAPE_DATA_HOME="$CAPTURE_DATA_HOME" \
        CAPTURE_CMD="$2" \
        script -qec 'stty cols 100 rows 50 -onlcr; exec bash -c "$CAPTURE_CMD"' /dev/null \
        </dev/null
}

# capture_prep <dir> <command>: set a project up for the captures, recording
# nothing. A failure prints the output and fails the build.
capture_prep() {
    local out
    out=$(_capture_run "$1" "$2") || {
        printf '%s\n' "\$ $2" "$out" >&2
        echo "capture_prep: the command above failed in $1" >&2
        return 1
    }
}

# capture [--exit <code>] <slug> <dir> <command>: run <command> in <dir> and
# append it to the record <slug>.
capture() {
    local expect=0
    if [ "$1" = --exit ]; then expect=$2 && shift 2; fi
    local slug=$1 cmd=$3 dir project out code=0 pair file="$OUT/$1.json"
    dir=$(cd "$2" && pwd -P) || return 1
    project=$(_capture_project "$dir") || return 1
    out=$(_capture_run "$dir" "$cmd") || code=$?
    out=${out//"$dir/"/}
    for pair in "${CAPTURE_PATHS[@]}"; do out=${out//"${pair%%=*}"/"${pair#*=}"}; done
    if [ "$code" != "$expect" ]; then
        printf '%s\n' "\$ $cmd" "$out" >&2
        echo "capture $slug: the command above exited $code, expected $expect" >&2
        return 1
    fi
    if [[ "$out" == *"$CAPTURE_TMP"* ]]; then
        printf '%s\n' "\$ $cmd" "$out" >&2
        echo "capture $slug: the output above names $CAPTURE_TMP; add a CAPTURE_PATHS pair" >&2
        return 1
    fi
    if [ -f "$file" ]; then
        jq --arg project "$project" --arg cmd "$cmd" --arg out "$out" --argjson code "$code" \
            'if .project == $project then .steps += [{$cmd, $out, $code}]
             else error("one record, two projects: \(.project) and \($project)") end' \
            "$file" >"$file.tmp" && mv "$file.tmp" "$file"
    else
        jq -n --arg project "$project" --arg cmd "$cmd" --arg out "$out" --argjson code "$code" \
            '{$project, steps: [{$cmd, $out, $code}]}' >"$file"
    fi
}

# capture_lines <slug> <dir>: capture each line of stdin as a step that exits
# 0. A quoted heredoc keeps each line exactly as the reader sees it.
capture_lines() {
    local line
    while IFS= read -r line; do
        capture "$1" "$2" "$line" || return 1
    done
}
