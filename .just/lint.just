set quiet := true
set unstable := true
set working-directory := '..'
set script-interpreter := ["bash", "-eo", "pipefail"]

[default]
[private]
list:
    @just --list lint

[doc("Check for unused dependencies")]
[group("gate")]
machete:
    cargo machete

[doc("Lint the GitHub workflows")]
[group("gate")]
workflows:
    actionlint

[doc("Lint every tracked shell script with shellcheck")]
[group("gate")]
[script]
shellcheck:
    git ls-files -z | xargs -0 awk 'FNR == 1 && /^#!.*[\/ ](ba)?sh( |$)/ { print FILENAME } { nextfile }' | xargs -r -d '\n' shellcheck --rcfile .config/shellcheckrc

[doc("Fail if any file exceeds its linecop cap")]
[group("gate")]
size:
    linecop

# The baseline keys a clone by a fingerprint of its raw tokens, so an edit
# inside an old clone makes it new. .config/jscpd.json, which jscpd reads from
# the repo root on every run here, skips declaration lines: `mod` lines with
# their attributes, `use` items and `("name", include_str!("..."))` registry
# entries. A list of those has no helper to extract, and one more line must not
# change a fingerprint. A skipped token also ends a clone that straddled it, so
# a new pattern re-mints the baseline once.
#
# The dry gate owns the clones at 50 tokens and up, renamed or similar. The
# repeats gate below owns the exact tokens from 15 to 49, where a pasted setup
# line lives. The two baselines stay apart: a 15-token fingerprint in the
# 50-token file would make the dry gate tolerate a clone it never measured.
#
# A tolerated clone is enforced by nothing, so every fingerprint in the
# baseline needs a reason in .config/jscpd-baseline-reasons.json. The check
# after jscpd fails on a fingerprint with no reason and on a reason with no
# fingerprint, so a re-mint cannot add a clone silently and a fix cannot leave
# its reason behind. The reasons file's shape and the house style are argued in
# .just/scripts/dry-reasons.sh.

DRY_FLAGS := "--no-tips -f rust -k 50 --ignore-identifiers --ignore-literals --similarity=0.8 --ignore '**/tests/it/main.rs' --baseline .config/jscpd-baseline.json"

# Fix a finding by extracting a shared helper, not by shuffling tokens until the
# detector stops matching. `--fail-on-new-clones 0` fails on a clone the
# baseline lacks; the report still lists every clone, so read the new ones by
# their fingerprint against `.config/jscpd-baseline.json`.

[doc("Copy-paste gate: fail on a duplicated Rust block the baseline lacks")]
[group("gate")]
dry:
    jscpd {{ DRY_FLAGS }} --fail-on-new-clones 0 -r ai crates
    bash .just/scripts/dry-reasons.sh

[doc("Re-mint .config/jscpd-baseline.json and require a reason for each tolerated clone")]
dry-baseline:
    jscpd {{ DRY_FLAGS }} --update-baseline -r silent crates
    bash .just/scripts/dry-reasons.sh

# The short-repeat gate reads jscpd's native `duplicates` pairs at 15 tokens,
# groups them by fragment, and fails on a group the baseline lacks. The bar,
# the gain formula and the reasons sidecar are argued in the script's header.
# `repeats-baseline` re-mints the baseline and refuses a group with no reason
# in .config/jscpd-repeats-reasons.json.

[doc("Short-repeat gate: fail on a 15-token repeat the baseline lacks")]
[group("gate")]
repeats:
    bash .just/scripts/repeats-gate.sh

[doc("Re-mint .config/jscpd-repeats-baseline.json and require a reason for each tolerated repeat")]
repeats-baseline:
    bash .just/scripts/repeats-gate.sh --mint

[doc("Fail on a decorative banner at the end of a Rust, Shell or CSS comment")]
[group("gate")]
banners:
    git ls-files --cached --others --exclude-standard -- '*.rs' '*.sh' '*.css' | xargs -r vale --no-global --config=.config/vale.ini --output=line

[doc("Fail on prose in Rust doc comments")]
[group("gate")]
vale-code:
    bash .just/scripts/check-code-prose.sh

[doc("Fail on a Vale level below error, a rule with no level, or a stale style link")]
[group("gate")]
vale-levels:
    bash .just/scripts/check-vale-levels.sh

[doc("Fail if the working tree carries a committed credential")]
[group("gate")]
secrets:
    bash .just/scripts/check-secrets.sh

# The control runs first, and the recipe runs it rather than the scanner,
# because the control drives the scanner. A control call inside the scanner
# would call itself.

[doc("Fail on a git conflict marker left in a tracked file")]
[group("gate")]
conflict-markers:
    bash .just/scripts/_conflict-markers-control.sh
    bash .just/scripts/check-conflict-markers.sh

[doc("Fail on a tracked file that names a setting on the reader's machine")]
[group("gate")]
host-details:
    bash .just/scripts/_host-details-control.sh
    bash .just/scripts/check-host-details.sh

[doc("Refuse a #[path] attribute that names anything but a test sibling")]
[group("gate")]
path-attr:
    bash .just/scripts/path-attr-check.sh

[doc("Fail on a broken or ambiguous intra-doc link")]
[group("gate")]
rustdoc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --quiet

[doc("Refuse task(s) and the like in a string outside tests; use mindtape::count")]
[group("gate")]
plural:
    bash .just/scripts/plural-check.sh

[doc("Refuse a user-facing string that names a recipe only this repo declares")]
[group("gate")]
no-just-in-output:
    bash .just/scripts/check-no-just-in-output.sh
