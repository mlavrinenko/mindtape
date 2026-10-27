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
DRY_FLAGS := "--no-tips -f rust -k 50 --ignore-identifiers --ignore-literals --similarity=0.8 --ignore '**/tests/it/main.rs' --baseline .config/jscpd-baseline.json"

# Fix a finding by extracting a shared helper, not by shuffling tokens until the
# detector stops matching. `--new-only` prints only the clones the baseline
# lacks.
[doc("Copy-paste gate: fail on a duplicated Rust block the baseline lacks")]
[group("gate")]
dry:
    jscpd {{ DRY_FLAGS }} --new-only --fail-on-new-clones 0 -r ai crates

[doc("Re-mint .config/jscpd-baseline.json after extracting a clone or accepting one")]
dry-baseline:
    jscpd {{ DRY_FLAGS }} --update-baseline -r silent crates

[doc("Fail if the working tree carries a committed credential")]
[group("gate")]
secrets:
    bash .just/scripts/check-secrets.sh

[doc("Refuse a #[path] attribute that names anything but a test sibling")]
[group("gate")]
path-attr:
    bash .just/scripts/path-attr-check.sh

[doc("Refuse task(s) and the like in a string outside tests; use mindtape::count")]
[group("gate")]
plural:
    bash .just/scripts/plural-check.sh
