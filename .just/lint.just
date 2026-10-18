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
    git ls-files -z | xargs -0 awk 'FNR == 1 && /^#!.*[\/ ](ba)?sh( |$)/ { print FILENAME } { nextfile }' | xargs -r -d '\n' shellcheck

[doc("Fail if any file exceeds its linecop cap")]
[group("gate")]
size:
    linecop

DRY_FLAGS := "--no-tips -f rust -k 50 --ignore-identifiers --ignore-literals --similarity=0.8 --ignore '**/tests/it/main.rs' --baseline .jscpd-baseline.json"

[doc("Copy-paste gate: fail on a duplicated Rust block the baseline lacks")]
[group("gate")]
dry:
    bash .just/scripts/lint-dry.sh {{ DRY_FLAGS }}

[doc("Re-mint .jscpd-baseline.json after extracting a clone or accepting one")]
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
