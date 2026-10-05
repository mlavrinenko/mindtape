#!/usr/bin/env bash
# .just/scripts/check-secrets.sh — the body of the `check-secrets` recipe
# (truth/tasks/scan-the-tree-for-secrets-with-gitleaks.typ). Runs from the
# repo root, like the recipe that invokes it.
#
# What it scans: the working tree as it stands, under `.gitleaks.toml` —
# gitleaks' own default ruleset plus this repo's allowlist, argued in that
# file, plus the walk bound derived below from what git ignores. History is out of scope; the reason
# is argued there too.
#
# Why the target is `.` and the CWD is the repo root, measured rather than
# assumed: gitleaks reports a finding's path exactly as its target was
# spelled, so `gitleaks dir "$PWD"` yields `/home/…/outdatty.lock` and every
# `^`-anchored allowlist in .gitleaks.toml silently stops matching — the
# gate would go from one known false positive to several, and the walk bound
# would stop bounding. A relative `.` from the root is the only invocation
# those anchors describe. It is also why arm one scans its fixture by cd-ing
# into it rather than by passing its path: a control that does not reproduce
# the real invocation pins the wrong thing.
#
# Arm one is a control, because this gate's failure mode is a false green and
# nothing else. "no leaks found" is what a clean tree prints, what an
# over-broad allowlist prints, and what a config that failed to load its rules
# prints. So one synthesized fixture, scanned through the real config before
# the real tree is, has to report an exact set of findings — and that single
# assertion pins six separate things at once:
#
#   - the default ruleset is live at all (`aws-access-token` fires),
#   - the derived walk bound fires and is `^`-anchored, and follows git's own
#     negation (the copies under `target/` and `.cargo/` are silent, the
#     tracked `.cargo/keep.toml` beside them is not),
#   - the outdatty.lock allowlist fires (the digest line is silent),
#   - it is scoped to the digest shape (a `generic-api-key` hit in that same
#     file that is not a bare sha256 is still reported — dropping
#     `condition = "AND"` from the config fails here and nowhere else),
#   - it is scoped to the rule (another rule's finding in that file survives),
#   - and no allowlist is broad enough to swallow the tree.
#
# It runs in a temp dir rather than in the tracked tree for size-fixture-
# check.sh's reason: `just check` runs its arms in parallel, and a real
# credential shape written under the repo would be read as real by arm two of
# a concurrent run.
set -eo pipefail

config=".gitleaks.toml"

if [ ! -f "$config" ]; then
    echo "check-secrets: $config is missing — running outside the repo root?" >&2
    exit 1
fi

# arm one

# Both literals carry `gitleaks:allow`, or arm two would report this file: it
# is a tracked file in the tree arm two scans, and the fixture below has to
# hold the bare credential shape for the control to mean anything. The
# comment is on the assignment only — what gets written to the fixture is the
# value, so the copy under the scanner is unannotated.
control_key="AKIAZZ4XK3PLQ2NR7WVB"                                              # gitleaks:allow
control_digest="0f3a9c1d7e2b48605af9c3d21e8b7460f5a2c9d31e4b8760a3f9c2d51e8b7463" # gitleaks:allow
control_nondigest="Zx91Qw82Mn73Bv64Ct55Dp46Er37Fg28Hi19Jk"                      # gitleaks:allow

repo="$PWD"
fixture=$(mktemp -d)
bounds=$(mktemp -d)
trap 'rm -rf "$fixture" "$bounds"' EXIT

# The walk bound is derived, never copied. gitleaks reads no ignore file
# (upstream gitleaks#1195), so an unbounded `gitleaks dir .` walks `target/`
# (1m44s over 2.09 GB, against 1.5s over 7.5 MB bounded) and every agent
# worktree under a globally ignored directory besides. git already knows the
# answer: `ls-files -oi --exclude-standard --directory` lists what it ignores
# across both .gitignore files, .git/info/exclude and core.excludesFile,
# collapses a wholly ignored directory to one entry, and never lists a
# tracked file, so `!/.cargo/mutants.toml` stays scanned without the
# lookahead RE2 lacks. Each entry becomes a `^<path>(/|$)` allowlist
# appended to a copy of $config: appended rather than `[extend] path`-ed,
# because a `targetRules` allowlist in an extended config is dropped (the
# outdatty.lock one, measured on gitleaks 8.30, and caught by arm one's
# control). `(/|$)` rather than `/`: `^gen/` never matches
# the directory `gen` itself, so the walk would descend and stat children a
# parallel docs arm may be deleting (measured under `--log-level debug` on
# gitleaks 8.30). The config is written outside the scanned tree.
derive_bound() {
    local dir=$1 out=$2 path
    local -a entries=()
    while IFS= read -r -d '' path; do
        path=$(sed 's/[][*.^$+?(){}|\\]/\\&/g' <<<"${path%/}")
        entries+=("  '''^$path(/|\$)''',")
    done < <(git -C "$dir" ls-files -z -oi --exclude-standard --directory)
    {
        cat "$repo/$config"
        if [ "${#entries[@]}" -gt 0 ]; then
            printf '\n[[allowlists]]\ndescription = "gitignored, derived by check-secrets.sh"\npaths = [\n'
            printf '%s\n' "${entries[@]}"
            printf ']\n'
        fi
    } >"$out"
}

# Resolved to an absolute path while `$PWD` is still the repo root: a shell
# with a direnv PATH hook (this repo's dev shell) rebuilds PATH off `$PWD` on
# every command, so a bare `gitleaks` looked up again after the `cd` below —
# arm one's control has to run from inside the fixture, not the repo root,
# for the same reason it cd's rather than passes a path (see above) — would
# silently fall back to whatever (if anything) sits outside the dev shell.
gitleaks_bin=$(command -v gitleaks) || {
    echo "check-secrets: gitleaks not found on PATH" >&2
    exit 1
}

mkdir -p "$fixture/target"
# Three lines, one per thing the outdatty.lock allowlist must get right. The
# first is a synthetic stand-in for the real lock's `refactor-auth.typ` entry
# — keyed on a name carrying "auth" so `generic-api-key` reads it as a secret
# — and must be silenced. Synthetic rather than copied, so re-recording that
# group cannot rot this fixture. The second is the same rule in the same file
# on a value that is not a bare sha256, and must survive: it is the only line
# that fails if the allowlist is widened from the digest shape to the whole
# file. The third is a different rule entirely, and must survive too.
cat >"$fixture/outdatty.lock" <<EOF
  refactor-auth.typ: $control_digest
  refactor-auth-key.typ: $control_nondigest
  leaked.typ: $control_key
EOF
# The bound's three cases, each decided by git's own rules: a copy under an
# ignored directory (silent), an ignored file beside a negated one (silent),
# and the negated file itself, tracked inside the ignored directory, which
# must still be reported.
mkdir -p "$fixture/.cargo"
printf 'key = %s\n' "$control_key" | tee "$fixture/target/leak.txt" \
    "$fixture/.cargo/junk" >"$fixture/.cargo/keep.toml"
printf '/target\n/.cargo/*\n!/.cargo/keep.toml\n' >"$fixture/.gitignore"
git -C "$fixture" init -q
git -C "$fixture" add .gitignore outdatty.lock .cargo/keep.toml
derive_bound "$fixture" "$bounds/control.toml"

# `--exit-code 0` so a finding is not an error here — the finding is the
# expected result, and the assertion is on the report rather than on the exit
# status. A gitleaks that fails for any other reason (an unparseable config,
# most likely) writes no report, which is caught separately below rather than
# left to `set -e` to end the script with no explanation.
report="$fixture/control.json"
if ! (cd "$fixture" && "$gitleaks_bin" dir -c "$bounds/control.toml" --no-banner --exit-code 0 \
    --report-format json --report-path "$report" . >"$fixture/control.log" 2>&1); then
    echo "check-secrets: gitleaks could not scan the control fixture" >&2
    sed 's/^/  /' "$fixture/control.log" >&2
    exit 1
fi

expected=".cargo/keep.toml:aws-access-token, outdatty.lock:aws-access-token, outdatty.lock:generic-api-key"
control=$(jq -r '[.[] | "\(.File):\(.RuleID)"] | sort | join(", ")' "$report")
if [ "$control" != "$expected" ]; then
    echo "check-secrets: the control fixture reported [$control]" >&2
    echo "  expected exactly [$expected]" >&2
    echo "  the scan below cannot be trusted — fix $config, not the fixture" >&2
    exit 1
fi

# arm two

# The scan holds target/www-generated.lock, because the walk bound in $config
# keeps this arm out of that directory's contents and cannot keep it out of
# the stat of the entry itself — the strace measurement, and why no pattern
# can buy that, are argued beside the bound in $config. www/generate.sh
# `rm -rf`s the directory on entry and `just check` is [parallel], so without
# the lock a deletion landing in that window logs an `ERR` on a run that
# finds no leaks and exits 0 (truth/tasks/check-secrets-still-logs-an-err-
# over-www-generated.typ). Same fd-9 flock www/generate.sh,
# docs/generate-md.sh and check-doc-facts.sh share — this arm is one more
# reader of that directory, and was the last one not taking it. No
# MINDTAPE_GENERATED_LOCK handshake: nothing holding the lock calls this
# gate, and an unconditional acquisition makes a future caller deadlock
# loudly rather than race quietly.
mkdir -p target
exec 9>target/www-generated.lock
flock 9

# Asserted rather than assumed — deleting the three lines above restores the
# race in silence, on a gate that stays green either way. `flock -n` opens
# the file on a NEW description, which conflicts with the one fd 9 holds, so
# it can only succeed if the lock is not ours.
if flock -n target/www-generated.lock true 2>/dev/null; then
    echo "check-secrets: the tree scan is not holding target/www-generated.lock" >&2
    echo "  restore the \`exec 9>…; flock 9\` above — without it a parallel docs" >&2
    echo "  arm deletes www/generated mid-walk and gitleaks ERRs on a green run" >&2
    exit 1
fi

# The report is read rather than the console output, and not for tidiness:
# gitleaks 8.30 prints `leaks found: N` and nothing else unless `-v` is
# passed, so a failing gate would name no file at all. Reading the JSON gives
# the file and line a reader has to act on — and gives them without the
# secret, which matters because this arm's output is what a failing CI job
# publishes. `--redact` covers gitleaks' own logging for the same reason.
#
# `--exit-code 0` for arm one's reason: a finding is not an error to be
# trapped here, it is the thing being measured, and it has to be
# distinguishable from gitleaks failing to run at all.
#
# `--log-level warn` drops the `INF scanned …`/`INF no leaks found` lines
# gitleaks prints on every run at its default "info" level — a real failure
# (an unparseable config, most likely) logs at FTL/ERR and still surfaces
# (verified against a malformed config file).
#
# The bound is derived here, under the lock, not before it: `ls-files` stats
# www/generated/ too, and a copy taken mid-`rm -rf` would leave the
# directory out of the bound for a walk that then descends into it.
derive_bound . "$bounds/tree.toml"
scan="$fixture/scan.json"
if ! gitleaks dir -c "$bounds/tree.toml" --no-banner --redact --exit-code 0 --log-level warn \
    --report-format json --report-path "$scan" .; then
    echo "check-secrets: gitleaks could not scan the working tree" >&2
    exit 1
fi

findings=$(jq -r '.[] | "  \(.File):\(.StartLine)  \(.RuleID)"' "$scan" | sort)
if [ -n "$findings" ]; then
    echo "check-secrets: gitleaks reported $(wc -l <<<"$findings") secret(s) in the working tree" >&2
    printf '%s\n' "$findings" >&2
    echo "  a real one is sops-encrypted or removed, never allowlisted" >&2
    echo "  a false positive is allowlisted in $config, scoped to the rule that" >&2
    echo "  misfired and to what it matched — never by fingerprint, which expires" >&2
    echo "  re-run \`gitleaks dir -c $config -v <file>\` to see what matched" >&2
    exit 1
fi

# Released the moment the walk is over: every [parallel] arm that touches
# www/generated/ is blocked while it is held, and neither arm below reads
# that directory — mmz filters its own walk by www/.gitignore.
exec 9>&-

# -------------------------------------------------------------- arm three

# The manifest coverage check lives in check-secrets-manifest.sh, its own file
# so this one clears the Shell cap; that file argues in place why it reads
# .mmz/config.yaml's own expansion and what it deliberately leaves uncovered.
# It reads nothing the lock above protects.
bash .just/scripts/check-secrets-manifest.sh
