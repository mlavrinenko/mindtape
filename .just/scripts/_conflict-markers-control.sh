#!/usr/bin/env bash
# .just/scripts/_conflict-markers-control.sh: the control the
# `lint::conflict-markers` recipe runs before check-conflict-markers.sh scans
# the real tree. Runs from the repo root. Builds synthetic repos in a temp
# dir, never the working tree: `just check` runs its arms in parallel, so a
# marker written under the repo would fail a concurrent run.
#
# A scan that finds nothing looks the same as a scanner that never opens a
# file. check-secrets.sh names that failure mode. "no leaks found" is what a
# clean tree prints, what an over-broad `ignore` array prints, and what a
# broken scanner prints. So the control drives the real scanner over fixtures
# and requires both answers. A clean repo passes, and each of the three
# marker lines fails with the file, the line and the marker in its message.
#
# It also pins the shapes that must not fire, because a gate that failed on a
# prose separator would get switched off. Those are `=======` with a trailing
# space, six `=`, `=======x`, an indented marker, and `<<<<<<<` or `>>>>>>>`
# with no trailing space. And it pins `git ls-files` as the file list: a
# marker in an untracked file passes.
#
# The recipe runs this before the scanner, not the scanner itself, because
# this script runs the scanner. A control call inside the scanner would call
# itself.
set -eo pipefail

scanner=$PWD/.just/scripts/check-conflict-markers.sh
if [ ! -f "$scanner" ]; then
    echo "check-conflict-markers: $scanner is missing — running outside the repo root?" >&2
    exit 1
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

fail() {
    echo "check-conflict-markers: control failed: $*" >&2
    echo "  the scan of the real tree proves nothing until the control passes" >&2
    exit 1
}

# expect <label> <repo> <want: 0|1> <needle>..., where `NOT:x` forbids.
expect() {
    local label=$1 d=$2 want=$3 rc=0 out needle
    shift 3
    out=$(cd "$d" && bash "$scanner" 2>&1) || rc=$?
    [ "$rc" -eq "$want" ] || fail "$label: exit $rc, want $want: $out"
    for needle in "$@"; do
        if [ "${needle#NOT:}" != "$needle" ]; then
            ! grep -qF -- "${needle#NOT:}" <<<"$out" || fail "$label: named ${needle#NOT:}: $out"
        else
            grep -qF -- "$needle" <<<"$out" || fail "$label: did not name $needle: $out"
        fi
    done
}

# repo <name>: a fresh git repo under the temp dir, printed. The caller
# stages every file it writes, so `git ls-files` lists it.
repo() {
    local d=$tmp/$1
    mkdir -p "$d"
    git -C "$d" init -q
    printf '%s' "$d"
}

# The clean fixture holds prose, a heading underline of eight `=`, and a
# marker quoted mid-line. None of them is a marker line.
clean=$(repo clean)
cat >"$clean/notes.md" <<'EOF'
# Notes

A paragraph.

========

The marker `<<<<<<< HEAD` opens a conflict, and `>>>>>>> main` closes it.
EOF
git -C "$clean" add notes.md
expect "clean repo" "$clean" 0

# Each marker line gets a repo of its own. The message must carry the file,
# the line and the marker, and the line number is the marker's own.
open=$(repo open)
printf 'one\ntwo\n<<<<<<< HEAD\nthree\n' >"$open/CHANGELOG.md"
git -C "$open" add CHANGELOG.md
expect "open marker" "$open" 1 \
    'error: CHANGELOG.md:3 has a conflict marker `<<<<<<<`' \
    'NOT:`=======`' 'NOT:`>>>>>>>`'

sep=$(repo sep)
printf 'one\n=======\ntwo\n' >"$sep/notes.md"
git -C "$sep" add notes.md
expect "separator" "$sep" 1 'error: notes.md:2 has a conflict marker `=======`'

close=$(repo close)
printf '>>>>>>> main\none\n' >"$close/notes.md"
git -C "$close" add notes.md
expect "close marker" "$close" 1 'error: notes.md:1 has a conflict marker `>>>>>>>`'

# The shapes that must not fire. A negative test whose fixture stays empty
# passes for the wrong reason (size-fixture-check.sh's lesson). So the
# fixture must hold each shape before the scan gets asked to stay silent.
near=$(repo near)
printf '======= \n======\n=======x\n<<<<<<<\n>>>>>>>\n  <<<<<<< HEAD\n' >"$near/notes.md"
for shape in '======= ' '======' '=======x' '<<<<<<<' '>>>>>>>' '  <<<<<<< HEAD'; do
    grep -qxF -- "$shape" "$near/notes.md" || fail "the near-miss fixture lacks [$shape]"
done
git -C "$near" add notes.md
expect "near misses" "$near" 0

# `git ls-files` supplies the file list: a marker in an untracked file
# passes, because the gate refuses what a commit would carry.
untracked=$(repo untracked)
printf 'one\n' >"$untracked/tracked.md"
printf '<<<<<<< HEAD\n' >"$untracked/untracked.md"
git -C "$untracked" add tracked.md
expect "untracked marker" "$untracked" 0

# A path with a space in it stays whole through `git ls-files -z` and
# `xargs -0`.
spaced=$(repo spaced)
printf 'one\n<<<<<<< HEAD\n' >"$spaced/a file.md"
git -C "$spaced" add 'a file.md'
expect "path with a space" "$spaced" 1 'error: a file.md:2 has a conflict marker `<<<<<<<`'

# The scan skips a tracked symlink: git stores the link text, not the file.
# The target stays untracked, so following the link would report `link.md:1`.
linked=$(repo linked)
printf '<<<<<<< HEAD\n' >"$linked/outside.md"
ln -s outside.md "$linked/link.md"
git -C "$linked" add link.md
expect "tracked symlink" "$linked" 0
