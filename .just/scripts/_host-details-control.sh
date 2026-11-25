#!/usr/bin/env bash
# .just/scripts/_host-details-control.sh: the control the `lint::host-details`
# recipe runs before check-host-details.sh scans the real tree. Runs from the
# repo root. Builds synthetic repos in a temp dir, never the working tree:
# `just check` runs its arms in parallel, so a setting written under the repo
# would fail a concurrent run.
#
# A scan that finds nothing looks the same as a scanner that never opens a
# file. check-secrets.sh names that failure mode. So the control drives the
# real scanner over fixtures and requires both answers. A clean repo passes,
# and each token fails with the file, the line and the token in its message.
#
# It also pins the shapes that must not fire, because a gate that failed on
# ordinary prose would get switched off. Those are `systemctl`, a token
# quoted mid-line in a fenced block, and a token in an untracked file.
#
# The recipe runs this before the scanner, not the scanner itself, because
# this script runs the scanner. A control call inside the scanner would call
# itself.
#
# Every expected message below is single-quoted, because the backticks around
# the token are message text and not a command substitution.
# shellcheck disable=SC2016
set -eo pipefail

scanner=$PWD/.just/scripts/check-host-details.sh
if [ ! -f "$scanner" ]; then
    echo "check-host-details: $scanner is missing — running outside the repo root?" >&2
    exit 1
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

fail() {
    echo "check-host-details: control failed: $*" >&2
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

# The clean fixture holds prose about the tools and a near-miss word. Neither
# is a host setting.
clean=$(repo clean)
cat >"$clean/notes.md" <<'EOF'
# Notes

callgrind and strace run unprivileged, so a recipe built on either works for a
normal user. `systemctl` is a different word and must not fire.
EOF
git -C "$clean" add notes.md
expect "clean repo" "$clean" 0

# Each token gets a repo of its own. The message must carry the file, the line
# and the token, and the line number is the token's own.
kernel=$(repo kernel)
printf 'one\ntwo\nset kernel.perf_event_paranoid to 1\nthree\n' >"$kernel/notes.md"
git -C "$kernel" add notes.md
expect "kernel setting" "$kernel" 1 \
    'error: notes.md:3 names a host setting `perf_event_paranoid`' \
    'NOT:`sysctl`'

sysctl=$(repo sysctl)
printf 'one\nrun sysctl -w kernel.perf_event_paranoid=1\ntwo\n' >"$sysctl/notes.md"
git -C "$sysctl" add notes.md
expect "sysctl" "$sysctl" 1 'error: notes.md:2 names a host setting `sysctl`'

proc=$(repo proc)
printf 'read /proc/sys/kernel/perf_event_paranoid\n' >"$proc/notes.md"
git -C "$proc" add notes.md
expect "proc path" "$proc" 1 'error: notes.md:1 names a host setting `/proc/sys/`'

cap=$(repo cap)
printf 'one\nsetcap cap_sys_admin+ep ./mt\n' >"$cap/notes.md"
git -C "$cap" add notes.md
expect "setcap" "$cap" 1 'error: notes.md:2 names a host setting `setcap`'

# `git ls-files` supplies the file list: a setting in an untracked file
# passes, because the gate refuses what a commit would carry.
untracked=$(repo untracked)
printf 'one\n' >"$untracked/tracked.md"
printf 'run sysctl -w kernel.perf_event_paranoid=1\n' >"$untracked/untracked.md"
git -C "$untracked" add tracked.md
expect "untracked setting" "$untracked" 0

# A path with a space in it stays whole through `git ls-files -z` and
# `xargs -0`.
spaced=$(repo spaced)
printf 'one\nrun sysctl -w kernel.perf_event_paranoid=1\n' >"$spaced/a file.md"
git -C "$spaced" add 'a file.md'
expect "path with a space" "$spaced" 1 'error: a file.md:2 names a host setting `sysctl`'

# The scan skips a tracked symlink: git stores the link text, not the file.
# The target stays untracked, so following the link would report `link.md:1`.
linked=$(repo linked)
printf 'run sysctl -w kernel.perf_event_paranoid=1\n' >"$linked/outside.md"
ln -s outside.md "$linked/link.md"
git -C "$linked" add link.md
expect "tracked symlink" "$linked" 0
