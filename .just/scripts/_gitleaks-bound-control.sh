#!/usr/bin/env bash
# .just/scripts/_gitleaks-bound-control.sh — the control fixture for
# .just/scripts/_gitleaks-bound-check.sh, run by check-secrets.sh immediately
# before the real pair (truth/tasks/hold-the-root-gitignore-to-the-gitleaks-
# walk-bound.typ).
#
# IT EXISTS FOR THE REASON check-secrets.sh's ARM ONE GIVES FOR ITS OWN: the
# thing it guards fails GREEN. "N gitignored paths covered, no tracked file
# hidden" is what a correct manifest prints, what a marker match loose enough
# to read its own documentation as an exemption prints — the check printed
# exactly that once, silently exempting `/.direnv`, the drift it was written
# for — and what a bound wide enough to match everything would print. A check
# whose pass and whose collapse are the same sentence proves nothing until
# something makes it speak.
#
# A SEPARATE FILE rather than a `--control` arm inside the check, for
# size-fixture-check.sh's reason: a self-test is its own subject, and folding
# it in put that file at 86% of the 300-line cap on the day it was created.
# _nav-diff.sh's extraction is the same shape one directory over — the
# extraction is what lets a fixture drive the logic on inputs the real tree
# cannot produce. No `just` recipe of its own, though: it asserts nothing a
# reader would run on purpose, and `just check-secrets` running it first is
# what makes the arm below trustworthy.
#
# THAT COST THE SAME HAND-WIRING, twice over, until it did not:
# `script-gitleaks-bound-control` and `script-gitleaks-bound-check` had to be
# named BY HAND in `just check-secrets`'s mmz inputs, because script-scope-
# check's arm two derived a rule's scopes from the recipe body it could see and
# neither of these is reached through `just`. Being the third and fourth such
# exemption is what finally turned that derivation transitive
# (truth/tasks/follow-script-to-script-calls-in-the-scope-derivation.typ). Both
# inputs are still there and neither is trusted: the walk reads check-secrets.sh
# reaching this file and fails that rule by name if its scopes stop covering it.
#
# FOUR ASSERTIONS ON ONE RUN, three loud and one silent, because each failure
# path can rot without touching another: arm one's miss, the unreasoned
# exemption, arm two's tracked-file hit, and the reasoned exemption staying
# quiet. The last is not decoration — a check that reports its own exemptions
# is one nobody can satisfy, and the obvious repair for that is to delete the
# exemption rather than the bug.
set -eo pipefail

check=".just/scripts/_gitleaks-bound-check.sh"
marker="gitleaks-bound: exempt"

if [ ! -f "$check" ]; then
    echo "gitleaks-bound-control: $check is missing — running outside the repo root?" >&2
    exit 1
fi

ctl=$(mktemp -d)
trap 'rm -rf "$ctl"' EXIT
mkdir -p "$ctl/fix"

# One pattern making both arms red at once: it covers nothing the fixture
# ignores, and it matches a file git tracks. The fixture .gitignore has to be
# named `.gitignore` and sit in its own directory — the check derives each
# entry's path prefix from that name and refuses any other.
printf "[[allowlists]]\npaths = ['^Cargo\\\\.toml\$']\n" >"$ctl/ctl.toml"
cat >"$ctl/fix/.gitignore" <<EOF
# $marker — a reason, which is what makes this entry pass in silence
/exempted
# $marker
/unreasoned
/demanded
EOF

# `bash "$check"`: nothing under .just/scripts/ carries an exec bit.
if out=$(bash "$check" "$ctl/ctl.toml" "$ctl/fix/.gitignore" 2>&1); then
    echo "gitleaks-bound-control: the control pair PASSED, so the check proves nothing" >&2
    sed 's/^/  /' <<<"$out" >&2
    exit 1
fi

for want in 'fix/demanded' 'exempts `unreasoned` with no reason' \
    'hides 1 TRACKED file' '  Cargo.toml'; do
    if ! grep -qF -- "$want" <<<"$out"; then
        echo "gitleaks-bound-control: the control pair did not report [$want]" >&2
        sed 's/^/  /' <<<"$out" >&2
        exit 1
    fi
done

if grep -qF -- 'exempted' <<<"$out"; then
    echo "gitleaks-bound-control: the control pair reported its REASONED exemption" >&2
    sed 's/^/  /' <<<"$out" >&2
    exit 1
fi
