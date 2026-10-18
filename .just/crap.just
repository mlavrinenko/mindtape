set quiet := true
set unstable := true
set working-directory := '..'
set script-interpreter := ["bash", "-eo", "pipefail"]

[default]
[private]
list:
    @just --list crap

CRAP_EXCLUDES := "--exclude **/main.rs --exclude **/ls/interactive/tui.rs --exclude **/*_tests.rs --exclude **/tests.rs --exclude **/*_tests_*.rs --exclude **/tests_*.rs"

[doc("Gate complex, undertested functions via the CRAP metric")]
[group("gate")]
check:
    bash .just/scripts/crap.sh all {{ CRAP_EXCLUDES }}

[doc("Re-mint crap-baseline.json after a new function or complexity rise")]
baseline:
    bash .just/scripts/crap-baseline.sh {{ CRAP_EXCLUDES }}

[doc("Fail if the CRAP baseline is empty or carries machine-local paths")]
[group("gate")]
baseline-check:
    bash .just/scripts/crap-baseline-check.sh
