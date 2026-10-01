# .just/scripts/_gate-drift-lib.sh — the two readers behind `just
# check-gate-drift`, sourced by check-gate-drift.sh and by the control that
# pins them (_gate-drift-control.sh). Why each reads what it reads is argued in
# check-gate-drift.sh. Both take a ROOT and read only under it, so the control
# can aim them at a synthetic tree and the gate at the real one.
#
# What a `check` arm REACHES is not decided here: _script-reach.sh walks it with
# the graph script-scope-check uses, and a second walker would learn the next
# shell form one at a time. This file keeps the two arm-specific parts.

reach_sh=$PWD/.just/scripts/_script-reach.sh

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
        # A `\` continuation joins to its first line, which names the offender.
        pend != "" { $2 = pend " " $2; $1 = pendwhere; pend = "" }
        $2 ~ /\\$/ { sub(/[ \t]*\\$/, "", $2); pend = $2; pendwhere = $1; next }
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

# build_drift <root>: judge every code line `check` reaches under <root>.
build_drift() {
    local root=$1 out
    out=$(cd "$root" && bash "$reach_sh" --also 'www/*.sh' --also 'docs/*.sh' \
        <(just --dump --dump-format json) check | find_plain_builds)
    [ -z "$out" ] && return 0
    echo "check-gate-drift: a gate-reachable build would relink target/debug/mt under the test arm" >&2
    sort -u <<<"$out" | awk -F'\t' '{ print "  " $1 ": " $3; print "    " $2 }' >&2
    echo "  see docs/contributing/gates.md, \"One mt binary, one feature set\"" >&2
    return 1
}
