# .just/scripts/_gate-drift-lib.sh — the two readers behind `just
# check-gate-drift`, sourced by check-gate-drift.sh and by the control that
# pins them (_gate-drift-control.sh). Why each reads what it reads is argued in
# check-gate-drift.sh; this file is the mechanism, kept apart so each script
# stays under the 300-line shell cap.
#
# Both take a ROOT and read only under it, so the control can aim them at a
# synthetic tree and the gate at the real one with the same code.

# arm 1 ----------------------------------------------------------------------

# toml_drift <root>: print each missing/extra path, exit 1 on any.
toml_drift() {
    local root=$1 listed known missing extra
    listed=$(cd "$root" && just --evaluate TOML_FMT_PATHS | tr -s '[:space:]' '\n' | sed '/^$/d' | sort -u)
    known=$(cd "$root" && git ls-files --cached --others --exclude-standard '*.toml' | sort -u)
    missing=$(comm -13 <(printf '%s\n' "$listed") <(printf '%s\n' "$known"))
    extra=$(comm -23 <(printf '%s\n' "$listed") <(printf '%s\n' "$known"))
    [ -z "$missing$extra" ] && return 0
    echo "check-gate-drift: TOML_FMT_PATHS differs from the TOML files git knows" >&2
    [ -z "$missing" ] || sed 's/^/  missing (git knows it, the list lacks it): /' <<<"$missing" >&2
    [ -z "$extra" ] || sed 's/^/  extra (the list names it, git does not know it): /' <<<"$extra" >&2
    echo "  fix: edit TOML_FMT_PATHS in the Justfile" >&2
    return 1
}

# arm 2 ----------------------------------------------------------------------

# Code lines of a script, as `where<TAB>line`: comments and heredoc bodies are
# data, and a `\` continuation joins to its first line's number.
file_lines() {
    awk -v file="$1" '
        function flush() { if (acc != "") print file ":" start "\t" acc; acc = "" }
        { sub(/^[ \t]+/, "") }
        heredoc != "" { if ($0 == heredoc) heredoc = ""; next }
        acc == "" && /^#/ { next }
        {
            if (acc == "") start = NR
            line = $0
            cont = (line ~ /\\$/)
            if (cont) sub(/[ \t]*\\$/, "", line)
            acc = (acc == "" ? line : acc " " line)
            if (cont) next
            if (match(acc, /<<-?[ \t]*['"'"'"]?[A-Za-z_][A-Za-z0-9_]*['"'"'"]?/)) {
                delim = substr(acc, RSTART, RLENGTH)
                sub(/^<<-?[ \t]*/, "", delim)
                gsub(/['"'"'"]/, "", delim)
                heredoc = delim
            }
            flush()
        }
        END { flush() }
    ' "$1"
}

# Body lines of one recipe out of the dump, same shape. just's `@`/`-` line
# prefixes are dropped; `{{ }}` fragments are dropped and the rest joined.
recipe_lines() {
    jq -r --arg want "$1" '
        def all_recipes:
          ((.recipes // {}) | to_entries | map(.value))
          + ((.modules // {}) | to_entries | map(.value) | map(all_recipes) | add // []);
        all_recipes[] | select(.namepath == $want) | .body // []
        | map(map(select(type == "string")) | join(" ")) | to_entries[]
        | "just \($want) body line \(.key + 1)\t\(.value)"
    ' <<<"$dump" | awk -F'\t' '
        function flush() { if (acc != "") print where "\t" acc; acc = "" }
        {
            line = $2
            sub(/^[ \t]+/, "", line)
            if (acc == "") { where = $1; sub(/^[@-]+/, "", line) }
            if (acc == "" && line ~ /^#/) next
            cont = (line ~ /\\$/)
            if (cont) sub(/[ \t]*\\$/, "", line)
            acc = (acc == "" ? line : acc " " line)
            if (!cont) flush()
        }
        END { flush() }
    '
}

# Stdin `where<TAB>line` -> one `where<TAB>reason<TAB>line` per cargo
# build/run of mindtape-cli that would write target/debug/mt without --tests.
find_plain_builds() {
    awk -F'\t' '
        function verdict(where, text,    n, i, w, sub_, tests, exempt, ws, pkgs, mt, tok, nt) {
            n = split(text, w, /[ \t]+/)
            i = 1
            envdir = 0
            while (i <= n && w[i] ~ /^[A-Za-z_][A-Za-z0-9_]*=/) {
                if (w[i] ~ /^CARGO_TARGET_DIR=/) envdir = 1
                i++
            }
            while (i <= n && w[i] ~ /^(exec|command|time|env|then|else|do)$/) i++
            if (w[i] != "cargo") return
            i++
            if (w[i] ~ /^\+/) i++
            sub_ = w[i]
            if (sub_ != "build" && sub_ != "b" && sub_ != "run" && sub_ != "r") return
            tests = 0; exempt = envdir; ws = 0; pkgs = 0; mt = 0
            for (i = i + 1; i <= n; i++) {
                tok = w[i]
                gsub(/["'"'"']/, "", tok)
                if (tok == "--") break
                if (tok == "--tests" && (sub_ == "build" || sub_ == "b")) tests = 1
                else if (tok == "--release" || tok == "-r" || tok ~ /^--target-dir(=|$)/) exempt = 1
                else if (tok == "--workspace" || tok == "--all") ws = 1
                else if (tok == "-p" || tok == "--package") {
                    nt = w[++i]; gsub(/["'"'"']/, "", nt); pkgs++
                    if (nt == "mindtape-cli") mt = 1
                } else if (tok ~ /^--package=/) {
                    pkgs++; sub(/^--package=/, "", tok)
                    if (tok == "mindtape-cli") mt = 1
                }
            }
            if (exempt || tests) return
            if (!(ws || pkgs == 0 || mt)) return
            if (sub_ == "run" || sub_ == "r")
                print where "\tcargo run builds mindtape-cli into target/debug/mt and takes no --tests; build with --tests and run ./target/debug/mt, or write elsewhere (--target-dir, --release)\t" text
            else
                print where "\tcargo build of mindtape-cli without --tests relinks target/debug/mt with another feature set; add --tests, or write elsewhere (--target-dir, --release)\t" text
        }
        {
            line = $2
            gsub(/[;&|(){}!]/, "\n", line)
            nl = split(line, frag, "\n")
            for (k = 1; k <= nl; k++) {
                f = frag[k]
                sub(/^[ \t]+/, "", f)
                if (f != "") verdict($1, f)
            }
        }
    '
}

# Stdin `where<TAB>line`, $1 the file or recipe it came from -> the nodes it
# RUNS: `r:<recipe>` for a nested `just` at command position, `f:<path>` for a
# script run as `bash x.sh`, `sh x.sh`, `source x.sh`, `. x.sh` or `./x.sh`. A
# script merely named (grepped, echoed in a hint) is read, not run, and reading
# it as an edge would fail a correct tree on a build the gate never reaches.
edges_of() {
    local from=$1 dir=. kind tok cand
    [ "${from#f:}" = "$from" ] || dir=$(dirname "${from#f:}")
    # awk finds the command-position words; bash only resolves the few distinct
    # candidates, so a script of hundreds of lines costs one process, not one
    # per line.
    awk -F'\t' '
        {
            line = $2
            gsub(/[;&|(){}!]/, "\n", line)
            nf = split(line, frag, "\n")
            for (k = 1; k <= nf; k++) {
                n = split(frag[k], w, /[ \t]+/)
                i = 1
                while (i <= n && (w[i] == "" || w[i] ~ /^[A-Za-z_][A-Za-z0-9_]*=/ || w[i] ~ /^(if|then|else|elif|do|while|until|time|exec|command|env)$/)) i++
                if (i > n) continue
                cmd = w[i++]
                if (cmd == "just") { print "r\t" w[i]; continue }
                if (cmd == "bash" || cmd == "sh" || cmd == "source" || cmd == ".") {
                    while (i <= n && w[i] ~ /^-/) i++
                    tok = w[i]
                } else if (cmd ~ /\.sh$/) tok = cmd
                else continue
                gsub(/["'"'"']/, "", tok)
                if (tok ~ /\.sh$/) print "f\t" tok
            }
        }
    ' | sort -u | while IFS=$'\t' read -r kind tok; do
        if [ "$kind" = r ]; then
            if [ -n "$tok" ] && [ -n "${recipes[$tok]}" ]; then echo "r:$tok"; fi
            continue
        fi
        for cand in "$tok" "$dir/$tok" "$dir/${tok##*/}"; do
            if [ -f "$cand" ]; then echo "f:${cand#./}"; break; fi
        done
    done
}

# build_drift <root>: walk what `check` reaches under <root>, print findings.
build_drift() {
    local root=$1 out
    out=$(cd "$root" && _build_drift_here) || return 1
    [ -z "$out" ] && return 0
    echo "check-gate-drift: a gate-reachable build would relink target/debug/mt under the test arm" >&2
    awk -F'\t' '{ print "  " $1 ": " $3; print "    " $2 }' <<<"$out" >&2
    echo "  see docs/contributing/gates.md, \"One mt binary, one feature set\"" >&2
    return 1
}

_build_drift_here() {
    dump=$(just --dump --dump-format json)
    declare -gA recipes=() seen=()
    local name node lines edge found=""
    while IFS= read -r name; do recipes[$name]=1; done < <(jq -r '
        def all_recipes:
          ((.recipes // {}) | to_entries | map(.value))
          + ((.modules // {}) | to_entries | map(.value) | map(all_recipes) | add // []);
        all_recipes[] | .namepath' <<<"$dump")
    # Seeds: what `check` passes to mmz, plus everything else it depends on.
    local queue=()
    while IFS= read -r name; do queue+=("r:$name"); done < <(jq -r '
        .recipes.check.dependencies[]? | if .recipe == "mmz" then .arguments[0] else .recipe end' <<<"$dump")
    if [ ${#queue[@]} -eq 0 ]; then
        echo "check-gate-drift: the dump has no \`check\` recipe with dependencies" >&2
        return 1
    fi
    while [ ${#queue[@]} -gt 0 ]; do
        node=${queue[0]}
        queue=("${queue[@]:1}")
        [ -z "${seen[$node]}" ] || continue
        seen[$node]=1
        if [ "${node#r:}" != "$node" ]; then
            lines=$(recipe_lines "${node#r:}")
            # Dependencies of a recipe run before it: a module's are siblings.
            local ns=${node#r:} prefix=""
            case $ns in *::*) prefix=${ns%::*}:: ;; esac
            while IFS= read -r name; do
                [ -z "$name" ] || queue+=("r:$prefix$name")
            done < <(jq -r --arg want "$ns" '
                def all_recipes:
                  ((.recipes // {}) | to_entries | map(.value))
                  + ((.modules // {}) | to_entries | map(.value) | map(all_recipes) | add // []);
                all_recipes[] | select(.namepath == $want) | .dependencies[]?
                | if .recipe == "mmz" then (.arguments[0] | select(. != null)) else .recipe end' <<<"$dump")
        else
            lines=$(file_lines "${node#f:}")
        fi
        [ -z "$lines" ] || found+=$(find_plain_builds <<<"$lines")$'\n'
        while IFS= read -r edge; do
            [ -z "$edge" ] || queue+=("$edge")
        done < <(edges_of "$node" <<<"$lines" | sort -u)
    done
    printf '%s' "$found" | sed '/^$/d' | sort -u
}

